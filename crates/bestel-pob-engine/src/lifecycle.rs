//! Process lifecycle: spawn, idle timeout, crash recovery, single-flight.
//!
//! One LuaJIT subprocess per game (PoE1 / PoE2 are independent). Calls are
//! serialised behind a `tokio::sync::Mutex`, so a single agent loop never
//! races itself. Idle for [`EngineConfig::idle_timeout`] → graceful `quit`,
//! kill if uncooperative. Crash > [`EngineConfig::max_restarts_per_window`]
//! within [`EngineConfig::restart_window`] → circuit-break for the rest of
//! the session.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::error::PobEngineError;
use crate::protocol::{decode, encode, Cmd, Reply};

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub luajit_path: PathBuf,
    pub harness_path: PathBuf,
    pub pob_root_poe1: PathBuf,
    pub pob_root_poe2: PathBuf,
    pub log_dir: PathBuf,
    pub idle_timeout: Duration,
    pub command_timeout: Duration,
    pub max_restarts_per_window: u32,
    pub restart_window: Duration,
}

impl EngineConfig {
    pub fn defaults(
        luajit_path: PathBuf,
        harness_path: PathBuf,
        pob_root_poe1: PathBuf,
        pob_root_poe2: PathBuf,
        log_dir: PathBuf,
    ) -> Self {
        Self {
            luajit_path,
            harness_path,
            pob_root_poe1,
            pob_root_poe2,
            log_dir,
            idle_timeout: Duration::from_secs(600),
            command_timeout: Duration::from_secs(8),
            max_restarts_per_window: 3,
            restart_window: Duration::from_secs(300),
        }
    }
}

pub(crate) struct Process {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    pub(crate) loaded_xml_hash: Option<u64>,
    pub(crate) last_activity: Instant,
}

impl Process {
    async fn send(&mut self, cmd: &Cmd, command_timeout: Duration) -> Result<Reply, PobEngineError> {
        let frame = encode(cmd)?;
        timeout(command_timeout, async {
            self.stdin.write_all(frame.as_bytes()).await?;
            self.stdin.flush().await?;
            let mut line = String::new();
            let n = self.stdout.read_line(&mut line).await?;
            if n == 0 {
                return Err(PobEngineError::EngineCrashed);
            }
            Ok::<_, PobEngineError>(line)
        })
        .await
        .map_err(|_| PobEngineError::Timeout(command_timeout))
        .and_then(|inner| inner)
        .and_then(|line| Ok(decode(&line)?))
        .and_then(|reply| {
            if !reply.ok {
                let msg = reply.error.clone().unwrap_or_else(|| "unknown error".into());
                return Err(PobEngineError::ProtocolBroken(msg));
            }
            self.last_activity = Instant::now();
            Ok(reply)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Game {
    Poe1,
    Poe2,
}

impl Game {
    pub fn as_str(self) -> &'static str {
        match self {
            Game::Poe1 => "poe1",
            Game::Poe2 => "poe2",
        }
    }
}

pub(crate) struct GameSlot {
    pub(crate) process: Option<Process>,
    pub(crate) restart_count: u32,
    pub(crate) restart_window_start: Instant,
    pub(crate) circuit_broken: bool,
}

impl GameSlot {
    fn new() -> Self {
        Self {
            process: None,
            restart_count: 0,
            restart_window_start: Instant::now(),
            circuit_broken: false,
        }
    }
}

pub struct PobEngineHandle {
    config: EngineConfig,
    poe1: Arc<Mutex<GameSlot>>,
    poe2: Arc<Mutex<GameSlot>>,
}

impl PobEngineHandle {
    pub fn new(config: EngineConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            poe1: Arc::new(Mutex::new(GameSlot::new())),
            poe2: Arc::new(Mutex::new(GameSlot::new())),
        })
    }

    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    pub(crate) fn slot(&self, game: Game) -> Arc<Mutex<GameSlot>> {
        match game {
            Game::Poe1 => self.poe1.clone(),
            Game::Poe2 => self.poe2.clone(),
        }
    }

    pub(crate) async fn ensure_alive<'a>(
        &self,
        game: Game,
        slot: &'a mut GameSlot,
    ) -> Result<&'a mut Process, PobEngineError> {
        if slot.circuit_broken {
            return Err(PobEngineError::CircuitBreaker(slot.restart_count));
        }

        let needs_spawn = if let Some(p) = slot.process.as_mut() {
            p.child.try_wait().ok().flatten().is_some()
        } else {
            true
        };

        if needs_spawn {
            // Reset window if it has elapsed.
            if slot.restart_window_start.elapsed() > self.config.restart_window {
                slot.restart_window_start = Instant::now();
                slot.restart_count = 0;
            }
            slot.restart_count = slot.restart_count.saturating_add(1);
            if slot.restart_count > self.config.max_restarts_per_window {
                slot.circuit_broken = true;
                return Err(PobEngineError::CircuitBreaker(slot.restart_count));
            }
            slot.process = Some(spawn(&self.config, game)?);
        }

        Ok(slot.process.as_mut().expect("process just ensured"))
    }

    pub(crate) async fn send_on(
        &self,
        proc: &mut Process,
        cmd: &Cmd,
    ) -> Result<Reply, PobEngineError> {
        proc.send(cmd, self.config.command_timeout).await
    }

    /// Cooperative shutdown: send `quit`, wait briefly, kill if needed.
    pub async fn shutdown(&self) {
        for slot in [self.poe1.clone(), self.poe2.clone()] {
            let mut s = slot.lock().await;
            if let Some(mut p) = s.process.take() {
                let _ = p.send(&Cmd::Quit, Duration::from_secs(1)).await;
                let _ = p.child.kill().await;
            }
        }
    }
}

fn spawn(config: &EngineConfig, game: Game) -> Result<Process, PobEngineError> {
    let pob_root = match game {
        Game::Poe1 => &config.pob_root_poe1,
        Game::Poe2 => &config.pob_root_poe2,
    };

    if !exists_safe(&config.luajit_path) {
        return Err(PobEngineError::Spawn(format!(
            "luajit binary missing at {}",
            config.luajit_path.display()
        )));
    }
    if !exists_safe(&config.harness_path) {
        return Err(PobEngineError::Spawn(format!(
            "api-stdio harness missing at {}",
            config.harness_path.display()
        )));
    }
    if !exists_safe(pob_root) {
        return Err(PobEngineError::Spawn(format!(
            "PoB root missing at {}",
            pob_root.display()
        )));
    }

    let _ = std::fs::create_dir_all(&config.log_dir);

    let mut cmd = Command::new(&config.luajit_path);
    cmd.arg(&config.harness_path)
        .arg(pob_root)
        .arg(game.as_str())
        .current_dir(pob_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Suppress the console window Windows would otherwise allocate when a
    // GUI-subsystem parent (release `bestel.exe` is `windows_subsystem =
    // "windows"`) spawns a console-subsystem child like `luajit.exe`. The
    // 0x0800_0000 flag is `CREATE_NO_WINDOW` from winbase.h. Without it
    // every chat or battery run pops a flicker of console windows on
    // screen — particularly bad for Tauri builds and for run-battery.
    cmd.kill_on_drop(true);

    #[cfg(windows)]
    {
        // `tokio::process::Command` exposes `creation_flags` directly on
        // Windows (no trait import needed — it mirrors std's CommandExt).
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| PobEngineError::Spawn(e.to_string()))?;

    let stdin = child.stdin.take().ok_or_else(|| {
        PobEngineError::Spawn("failed to capture sidecar stdin".into())
    })?;
    let stdout = child.stdout.take().ok_or_else(|| {
        PobEngineError::Spawn("failed to capture sidecar stdout".into())
    })?;

    if let Some(stderr) = child.stderr.take() {
        let pid = child.id().unwrap_or(0);
        let log_path = config
            .log_dir
            .join(format!("{}-{}.log", game.as_str(), pid));
        tokio::spawn(drain_stderr(stderr, log_path));
    }

    Ok(Process {
        child,
        stdin,
        stdout: BufReader::new(stdout),
        loaded_xml_hash: None,
        last_activity: Instant::now(),
    })
}

fn exists_safe(p: &Path) -> bool {
    std::fs::metadata(p).is_ok()
}

async fn drain_stderr(mut stderr: tokio::process::ChildStderr, log_path: PathBuf) {
    use tokio::io::AsyncReadExt;
    let mut buf = vec![0u8; 4096];
    let mut sink: Option<tokio::fs::File> = None;
    loop {
        match stderr.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                if sink.is_none() {
                    sink = tokio::fs::File::create(&log_path).await.ok();
                }
                if let Some(f) = sink.as_mut() {
                    let _ = f.write_all(&buf[..n]).await;
                }
            }
        }
    }
}
