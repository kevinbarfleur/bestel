//! Event loop architecture
//!
//! ```text
//!     crossterm EventStream  ──────────► [input_task]  ──────────►┐
//!     PoB watcher                                                  │
//!         broadcast::Receiver<PobBuild>  ──────────────────────────┤
//!                                                                  ├──► run_loop ──► AppState ──► ui::render
//!     LLM provider task (Anthropic/Codex/Claude)                   │
//!         mpsc::UnboundedSender<LlmDelta>  ────────────────────────┤
//!     tick interval (every 120ms while animating)                  ┘
//! ```
//!
//! Each input source feeds the same `AppEvent` enum via dedicated channels.
//! The crossterm reader runs in its OWN task so the OS-level pipe is always
//! drained promptly — even when render or LLM work is busy. Without this we
//! observed lost keystrokes on Windows when streaming filled the LLM channel.
//!
//! Redraw policy:
//! - Always redraw on key, mouse, PoB and LLM events (immediate user feedback).
//! - Tick events redraw only if at least 33 ms passed since the last render
//!   (drives the spinner without burning CPU).
//! - Idle ticks emit no redraw (preserves terminal text selection).

use std::collections::HashMap;
use std::io::{stdout, Stdout, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEvent,
    KeyEventKind, KeyModifiers, KeyboardEnhancementFlags, MouseEventKind,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures_util::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::{broadcast, mpsc, oneshot};

use bestel_core::devlog;
use bestel_core::llm::detect::{detect_provider, render_probes, Probe};
use bestel_core::llm::tools::BuildContext;
use bestel_core::llm::{ChatMessage, LlmDelta, Provider, Role, ToolStatus};
use bestel_core::pob::watcher::PobWatcher;
use bestel_core::pob::PobBuild;
use serde_json::json;

use crate::items::ChatItem;
use crate::ui;

pub struct AppState {
    pub items: Vec<ChatItem>,
    pub history: Vec<ChatMessage>,
    pub input: String,
    pub streaming: bool,
    pub spinner_frame: usize,
    pub follow_tail: bool,
    pub scroll: u16,
    pub current_assistant_idx: Option<usize>,
    pub current_reasoning_idx: Option<usize>,
    pub current_tool_idx: HashMap<String, usize>,
    pub cancel_tx: Option<oneshot::Sender<()>>,
    pub build: Option<PobBuild>,
    pub model_label: String,
    pub auth_label: String,
    pub probes: Vec<Probe>,
    pub watching_dirs: Vec<String>,
    pub status: String,
    pub last_render_at: Option<Instant>,
}

impl AppState {
    fn new(
        model_label: String,
        auth_label: String,
        probes: Vec<Probe>,
        watching_dirs: Vec<String>,
    ) -> Self {
        Self {
            items: vec![ChatItem::System(
                "Bestel observe les vieux récits. Pose ta question, exilé.".into(),
            )],
            history: Vec::new(),
            input: String::new(),
            streaming: false,
            spinner_frame: 0,
            follow_tail: true,
            scroll: 0,
            current_assistant_idx: None,
            current_reasoning_idx: None,
            current_tool_idx: HashMap::new(),
            cancel_tx: None,
            build: None,
            model_label,
            auth_label,
            probes,
            watching_dirs,
            status: String::new(),
            last_render_at: None,
        }
    }

    pub fn push_item(&mut self, item: ChatItem) -> usize {
        self.items.push(item);
        self.items.len() - 1
    }

    fn handle_delta(&mut self, d: LlmDelta) {
        log_delta(&d);
        match d {
            LlmDelta::TextDelta(text) => {
                let idx = match self.current_assistant_idx {
                    Some(i) => i,
                    None => {
                        let i = self.push_item(ChatItem::Assistant {
                            text: String::new(),
                            complete: false,
                            started: Instant::now(),
                        });
                        self.current_assistant_idx = Some(i);
                        i
                    }
                };
                if let Some(ChatItem::Assistant { text: t, .. }) = self.items.get_mut(idx) {
                    t.push_str(&text);
                }
            }
            LlmDelta::ReasoningBegin => {
                if self.current_reasoning_idx.is_none() {
                    let idx = self.push_item(ChatItem::Reasoning {
                        summary: String::new(),
                        complete: false,
                        expanded: true,
                        started: Instant::now(),
                        ended: None,
                    });
                    self.current_reasoning_idx = Some(idx);
                }
            }
            LlmDelta::ReasoningDelta(s) => {
                if self.current_reasoning_idx.is_none() {
                    let idx = self.push_item(ChatItem::Reasoning {
                        summary: String::new(),
                        complete: false,
                        expanded: true,
                        started: Instant::now(),
                        ended: None,
                    });
                    self.current_reasoning_idx = Some(idx);
                }
                if let Some(idx) = self.current_reasoning_idx {
                    if let Some(ChatItem::Reasoning { summary, .. }) = self.items.get_mut(idx) {
                        summary.push_str(&s);
                    }
                }
            }
            LlmDelta::ReasoningEnd => {
                if let Some(idx) = self.current_reasoning_idx.take() {
                    if let Some(ChatItem::Reasoning {
                        complete, ended, ..
                    }) = self.items.get_mut(idx)
                    {
                        *complete = true;
                        *ended = Some(Instant::now());
                    }
                }
            }
            LlmDelta::ToolBegin { id, name, detail } => {
                let idx = self.push_item(ChatItem::Tool {
                    id: id.clone(),
                    name,
                    detail,
                    output: String::new(),
                    status: ToolStatus::Running,
                    expanded: true,
                    started: Instant::now(),
                    ended: None,
                });
                self.current_tool_idx.insert(id, idx);
            }
            LlmDelta::ToolOutput { id, chunk } => {
                if let Some(&idx) = self.current_tool_idx.get(&id) {
                    if let Some(ChatItem::Tool { output, .. }) = self.items.get_mut(idx) {
                        if !output.is_empty() {
                            output.push('\n');
                        }
                        output.push_str(&chunk);
                    }
                }
            }
            LlmDelta::ToolEnd {
                id,
                status,
                summary,
            } => {
                if let Some(idx) = self.current_tool_idx.remove(&id) {
                    if let Some(ChatItem::Tool {
                        status: s,
                        ended,
                        detail,
                        output,
                        ..
                    }) = self.items.get_mut(idx)
                    {
                        *s = status;
                        *ended = Some(Instant::now());
                        if let Some(sum) = summary {
                            let is_placeholder = detail
                                .as_deref()
                                .map(|d| {
                                    d.is_empty()
                                        || d.starts_with('(')
                                        || d.contains("\"query\":\"\"")
                                })
                                .unwrap_or(true);
                            if is_placeholder && !sum.is_empty() {
                                *detail = Some(sum.clone());
                            } else if !sum.is_empty() && !output.contains(&sum) {
                                if !output.is_empty() {
                                    output.push('\n');
                                }
                                output.push_str(&sum);
                            }
                        }
                    }
                }
            }
            LlmDelta::MessageEnd => {
                if let Some(idx) = self.current_assistant_idx.take() {
                    if let Some(ChatItem::Assistant {
                        text, complete, ..
                    }) = self.items.get_mut(idx)
                    {
                        *complete = true;
                        let final_text = text.clone();
                        if !final_text.is_empty() {
                            devlog::log_value(
                                "assistant_final",
                                json!({ "text": final_text, "len": final_text.chars().count() }),
                            );
                            self.history.push(ChatMessage {
                                role: Role::Assistant,
                                content: final_text,
                            });
                        }
                    }
                }
                self.streaming = false;
                self.cancel_tx = None;
                self.status.clear();
            }
            LlmDelta::Error(msg) => {
                self.streaming = false;
                self.cancel_tx = None;
                self.current_assistant_idx = None;
                self.current_reasoning_idx = None;
                self.current_tool_idx.clear();
                self.push_item(ChatItem::System(format!("⚠ {}", msg)));
                self.status = format!("Erreur : {}", msg);
            }
        }
    }
}

enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEventKind),
    Resize,
    Pob(PobBuild),
    Llm(LlmDelta),
    Tick,
}

pub async fn run() -> Result<()> {
    let watcher = PobWatcher::start()?;
    let watching_dirs: Vec<String> = watcher
        .installs
        .iter()
        .map(|i| format!("{}: {}", i.game.label(), i.builds_dir.display()))
        .collect();

    let initial_build = watcher.initial_build();
    let mut pob_rx: broadcast::Receiver<PobBuild> = watcher.subscribe();

    let ctx = BuildContext::new();
    if let Some(b) = initial_build.clone() {
        ctx.set(b);
    }

    let detection = detect_provider().await;
    let (model_label, auth_label) = match &detection.provider {
        Some(p) => (p.label(), p.auth_label().to_string()),
        None => ("aucun provider".into(), "—".into()),
    };
    let probes = detection.probes.clone();
    let provider = detection.provider.map(Arc::new);

    let mut state = AppState::new(model_label, auth_label, probes, watching_dirs);
    state.build = initial_build;

    if provider.is_none() {
        let probes_txt = render_probes(&state.probes);
        let mut msg = String::from(
            "Aucun moyen de parler avec Bestel. Installe l'un de ces outils \
             ou définis ANTHROPIC_API_KEY :\n",
        );
        msg.push_str(&probes_txt);
        state.push_item(ChatItem::System(msg));
    } else {
        let probes_txt = render_probes(&state.probes);
        state.push_item(ChatItem::System(format!(
            "Provider actif : {} ({}).\n{}",
            state.model_label, state.auth_label, probes_txt
        )));
    }

    if devlog::is_enabled() {
        if let Some(p) = devlog::log_path() {
            state.push_item(ChatItem::System(format!(
                "🪵 Dev log actif → {}",
                p.display()
            )));
        }
        devlog::log_value(
            "session_start",
            json!({
                "provider": state.model_label,
                "auth": state.auth_label,
                "build": state.build.as_ref().map(|b| b.summary_line()),
            }),
        );
    }

    let kitty_supported = setup_terminal_pre()?;
    let mut terminal = build_terminal();
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<AppEvent>();
    spawn_input_task(input_tx);

    let (llm_tx, mut llm_rx) = mpsc::unbounded_channel::<LlmDelta>();

    let result = run_loop(
        &mut terminal,
        &mut state,
        &mut input_rx,
        &mut pob_rx,
        &mut llm_rx,
        &llm_tx,
        provider.clone(),
        ctx.clone(),
    )
    .await;

    teardown_terminal(&mut terminal, kitty_supported)?;
    result
}

fn spawn_input_task(tx: mpsc::UnboundedSender<AppEvent>) {
    tokio::spawn(async move {
        let mut events = EventStream::new();
        while let Some(maybe) = events.next().await {
            let Ok(ev) = maybe else { continue };
            let app_ev = match ev {
                Event::Key(k) => {
                    // Log EVERY raw key the terminal sends us (even Release).
                    // Critical for diagnosing 'modifier swallowed by terminal'
                    // issues — the user can press a combo and see exactly
                    // what crossterm received. Disabled at runtime unless
                    // BESTEL_DEV_LOG=1.
                    devlog::log_value(
                        "raw_key",
                        json!({
                            "code": format!("{:?}", k.code),
                            "modifiers": format!("{:?}", k.modifiers),
                            "kind": format!("{:?}", k.kind),
                            "state": format!("{:?}", k.state),
                        }),
                    );
                    // Accept Press AND Repeat. On Windows, holding a key
                    // fires Repeat events; without this branch they are
                    // silently dropped and typing feels lossy.
                    if matches!(k.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                        AppEvent::Key(k)
                    } else {
                        continue;
                    }
                }
                Event::Mouse(m) => AppEvent::Mouse(m.kind),
                Event::Resize(_, _) => AppEvent::Resize,
                _ => continue,
            };
            if tx.send(app_ev).is_err() {
                break;
            }
        }
    });
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut AppState,
    input_rx: &mut mpsc::UnboundedReceiver<AppEvent>,
    pob_rx: &mut broadcast::Receiver<PobBuild>,
    llm_rx: &mut mpsc::UnboundedReceiver<LlmDelta>,
    llm_tx: &mpsc::UnboundedSender<LlmDelta>,
    provider: Option<Arc<Provider>>,
    ctx: BuildContext,
) -> Result<()> {
    draw(terminal, state)?;

    let mut tick = tokio::time::interval(Duration::from_millis(120));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        let app_event: AppEvent = tokio::select! {
            biased;
            // bias: prioritise user input over LLM/tick so typing always
            // feels responsive even during heavy streaming.
            Some(ev) = input_rx.recv() => ev,
            res = pob_rx.recv() => match res {
                Ok(b) => AppEvent::Pob(b),
                Err(_) => continue,
            },
            Some(d) = llm_rx.recv() => AppEvent::Llm(d),
            _ = tick.tick() => AppEvent::Tick,
        };

        let mut needs_redraw = true;
        let is_tick = matches!(app_event, AppEvent::Tick);

        match app_event {
            AppEvent::Key(k) => {
                if handle_key(state, k, &provider, &ctx, llm_tx) {
                    return Ok(());
                }
            }
            AppEvent::Mouse(MouseEventKind::ScrollUp) => {
                state.scroll = state.scroll.saturating_add(3);
                state.follow_tail = false;
            }
            AppEvent::Mouse(MouseEventKind::ScrollDown) => {
                state.scroll = state.scroll.saturating_sub(3);
                if state.scroll == 0 {
                    state.follow_tail = true;
                }
            }
            AppEvent::Mouse(_) => {
                needs_redraw = false;
            }
            AppEvent::Resize => {}
            AppEvent::Pob(b) => {
                ctx.set(b.clone());
                state.status = format!("PoB mis à jour : {}", b.summary_line());
                state.build = Some(b);
            }
            AppEvent::Llm(d) => state.handle_delta(d),
            AppEvent::Tick => {
                if state.streaming || has_running_item(state) {
                    state.spinner_frame = state.spinner_frame.wrapping_add(1);
                } else {
                    needs_redraw = false;
                }
            }
        }

        if state.input.is_empty()
            && state.cancel_tx.is_none()
            && !state.streaming
            && state.status == "Annulé."
        {
            // Nothing to do — placeholder for future cleanup hooks.
        }

        if needs_redraw {
            let throttle_ok = state
                .last_render_at
                .map(|t| t.elapsed() >= Duration::from_millis(33))
                .unwrap_or(true);
            // Force redraw on real events; ticks respect throttling.
            let must = !is_tick || throttle_ok;
            if must {
                draw(terminal, state)?;
                state.last_render_at = Some(Instant::now());
            }
        }
    }
}

/// Handle a single key press / repeat event.
///
/// Newline insertion in the input field is supported via three keys, so it
/// works regardless of which keyboard protocol is active:
///   - Shift+Enter (kitty keyboard protocol; only on supporting terminals)
///   - Alt+Enter   (works almost universally — escape sequence with no ambiguity)
///   - Ctrl+J      (literal LF; works on every POSIX-style terminal)
///
/// Returns `true` when the app should quit.
fn handle_key(
    state: &mut AppState,
    k: KeyEvent,
    provider: &Option<Arc<Provider>>,
    ctx: &BuildContext,
    llm_tx: &mpsc::UnboundedSender<LlmDelta>,
) -> bool {
    let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
    let shift = k.modifiers.contains(KeyModifiers::SHIFT);
    let alt = k.modifiers.contains(KeyModifiers::ALT);

    if ctrl && k.code == KeyCode::Char('c') {
        return true;
    }

    match k.code {
        KeyCode::Esc => {
            if state.streaming {
                cancel_streaming(state);
            } else {
                return true;
            }
        }
        // Newline triggers — listed broadly so at least one works on every
        // terminal/host combo. The original Enter without any modifier
        // submits the message.
        KeyCode::Enter if shift || alt || ctrl => {
            if !state.streaming {
                state.input.push('\n');
            }
        }
        KeyCode::Char('j') if ctrl => {
            if !state.streaming {
                state.input.push('\n');
            }
        }
        KeyCode::Char('m') if ctrl && shift => {
            if !state.streaming {
                state.input.push('\n');
            }
        }
        KeyCode::Char('\n') => {
            // Some terminals decode Ctrl+J as a literal LF char.
            if !state.streaming {
                state.input.push('\n');
            }
        }
        KeyCode::Enter => {
            if !state.streaming {
                submit(state, provider.clone(), ctx.clone(), llm_tx.clone());
            }
        }
        KeyCode::Backspace => {
            if !state.streaming {
                state.input.pop();
            }
        }
        KeyCode::Char(c) => {
            // Ignore Ctrl+<letter> combos other than the ones we handle —
            // they should not insert text.
            if ctrl {
                return false;
            }
            if !state.streaming {
                state.input.push(c);
            }
        }
        KeyCode::PageUp => {
            state.scroll = state.scroll.saturating_add(10);
            state.follow_tail = false;
        }
        KeyCode::PageDown => {
            state.scroll = state.scroll.saturating_sub(10);
            if state.scroll == 0 {
                state.follow_tail = true;
            }
        }
        KeyCode::Up => {
            state.scroll = state.scroll.saturating_add(1);
            state.follow_tail = false;
        }
        KeyCode::Down => {
            state.scroll = state.scroll.saturating_sub(1);
            if state.scroll == 0 {
                state.follow_tail = true;
            }
        }
        KeyCode::Home => {
            state.scroll = u16::MAX;
            state.follow_tail = false;
        }
        KeyCode::End => {
            state.scroll = 0;
            state.follow_tail = true;
        }
        _ => {}
    }
    false
}

fn submit(
    state: &mut AppState,
    provider: Option<Arc<Provider>>,
    ctx: BuildContext,
    tx: mpsc::UnboundedSender<LlmDelta>,
) {
    let text = state.input.trim().to_string();
    if text.is_empty() {
        return;
    }
    state.input.clear();
    devlog::log_value("user_input", json!({ "text": text }));
    state.push_item(ChatItem::User(text.clone()));
    state.history.push(ChatMessage {
        role: Role::User,
        content: text,
    });
    state.follow_tail = true;
    let Some(p) = provider else {
        state.status = "Aucun provider disponible.".into();
        return;
    };
    state.streaming = true;
    state.status = "Bestel consulte les vieux récits…".into();
    let history = state.history.clone();
    let (cancel_tx, cancel_rx) = oneshot::channel();
    state.cancel_tx = Some(cancel_tx);
    tokio::spawn(async move {
        tokio::select! {
            res = p.run(history, ctx, tx.clone()) => {
                if let Err(e) = res {
                    let _ = tx.send(LlmDelta::Error(e.to_string()));
                }
            }
            _ = cancel_rx => {
                // Provider future is dropped here; subprocess kill_on_drop
                // handles the OS-level cleanup.
            }
        }
    });
}

fn cancel_streaming(state: &mut AppState) {
    if let Some(tx) = state.cancel_tx.take() {
        let _ = tx.send(());
    }
    state.streaming = false;
    state.status = "Annulé.".into();
    if let Some(idx) = state.current_assistant_idx.take() {
        if let Some(ChatItem::Assistant {
            complete, text, ..
        }) = state.items.get_mut(idx)
        {
            *complete = true;
            if !text.is_empty() {
                state.history.push(ChatMessage {
                    role: Role::Assistant,
                    content: text.clone(),
                });
            }
        }
    }
    if let Some(idx) = state.current_reasoning_idx.take() {
        if let Some(ChatItem::Reasoning {
            complete, ended, ..
        }) = state.items.get_mut(idx)
        {
            *complete = true;
            *ended = Some(Instant::now());
        }
    }
    for (_id, idx) in std::mem::take(&mut state.current_tool_idx) {
        if let Some(ChatItem::Tool { status, ended, .. }) = state.items.get_mut(idx) {
            *status = ToolStatus::Failed;
            *ended = Some(Instant::now());
        }
    }
}

fn log_delta(d: &LlmDelta) {
    if !devlog::is_enabled() {
        return;
    }
    let (kind, payload) = match d {
        LlmDelta::TextDelta(t) => ("text_delta", json!({ "len": t.chars().count(), "text": t })),
        LlmDelta::ReasoningBegin => ("reasoning_begin", json!({})),
        LlmDelta::ReasoningDelta(t) => (
            "reasoning_delta",
            json!({ "len": t.chars().count(), "text": t }),
        ),
        LlmDelta::ReasoningEnd => ("reasoning_end", json!({})),
        LlmDelta::ToolBegin { id, name, detail } => (
            "tool_begin",
            json!({ "id": id, "name": name, "detail": detail }),
        ),
        LlmDelta::ToolOutput { id, chunk } => {
            ("tool_output", json!({ "id": id, "chunk": chunk }))
        }
        LlmDelta::ToolEnd {
            id,
            status,
            summary,
        } => (
            "tool_end",
            json!({
                "id": id,
                "status": format!("{:?}", status),
                "summary": summary,
            }),
        ),
        LlmDelta::MessageEnd => ("message_end", json!({})),
        LlmDelta::Error(msg) => ("error", json!({ "message": msg })),
    };
    devlog::log_delta("tui", kind, payload);
}

fn has_running_item(state: &AppState) -> bool {
    state.items.iter().any(|it| match it {
        ChatItem::Tool { status, .. } => *status == ToolStatus::Running,
        ChatItem::Reasoning { complete, .. } => !*complete,
        ChatItem::Assistant { complete, .. } => !*complete,
        _ => false,
    })
}

fn draw(terminal: &mut Terminal<CrosstermBackend<Stdout>>, state: &AppState) -> Result<()> {
    terminal.draw(|f| ui::render(f, state))?;
    Ok(())
}

/// Returns true if the kitty keyboard protocol was successfully pushed.
///
/// We enable THREE separate keyboard-extension protocols, in order of preference,
/// so that Shift+Enter / Ctrl+Enter / Alt+letter combos are properly reported as
/// distinct events on every host. This mirrors what Claude Code, OpenCode, and
/// other modern TUIs do — relying on a single protocol is fragile.
fn setup_terminal_pre() -> Result<bool> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;

    // 1. Kitty keyboard protocol — used by kitty, foot, alacritty 0.13+,
    //    Ghostty, WezTerm, Windows Terminal 1.21+. The terminal reports
    //    every key as an escape code with full modifier info.
    let kitty_ok = execute!(
        out,
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS,
        )
    )
    .is_ok();

    // 2. xterm modifyOtherKeys level 2 — supported by xterm, urxvt,
    //    Windows Terminal (all recent versions), alacritty, kitty as a
    //    fallback. Reports Shift+Enter, Ctrl+Enter etc. as
    //    `CSI 27;<mod>;<keycode>~` sequences which crossterm decodes.
    //    This is what makes Shift+Enter work on Windows Terminal even
    //    without kitty protocol support.
    let _ = out.write_all(b"\x1b[>4;2m");

    // 3. Win32 input mode — Windows Terminal 1.4+ specific. Forces the
    //    terminal to emit detailed virtual-key sequences instead of the
    //    legacy ANSI codes, so modifiers on every key (including Enter)
    //    are preserved. Harmless no-op on non-Windows hosts.
    #[cfg(windows)]
    let _ = out.write_all(b"\x1b[?9001h");

    let _ = out.flush();
    Ok(kitty_ok)
}

fn build_terminal() -> Terminal<CrosstermBackend<Stdout>> {
    let backend = CrosstermBackend::new(stdout());
    Terminal::new(backend).expect("terminal init")
}

fn teardown_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    kitty: bool,
) -> Result<()> {
    let backend = terminal.backend_mut();
    // Reverse order of setup_terminal_pre.
    #[cfg(windows)]
    let _ = backend.write_all(b"\x1b[?9001l");
    let _ = backend.write_all(b"\x1b[>4m");
    let _ = backend.flush();
    if kitty {
        let _ = execute!(backend, PopKeyboardEnhancementFlags);
    }
    let _ = execute!(backend, DisableMouseCapture, LeaveAlternateScreen);
    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}
