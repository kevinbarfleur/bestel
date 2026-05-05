use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::{broadcast, mpsc};

use super::locator::{find_pob_dirs, most_recent_build, PobInstall};
use super::parser::{parse_file, parse_summary};
use super::{PobBuild, PobBuildSummary};

pub struct PobWatcher {
    pub installs: Vec<PobInstall>,
    pub events: broadcast::Sender<PobBuild>,
    _keepalive: Vec<RecommendedWatcher>,
}

impl PobWatcher {
    pub fn start() -> Result<Self> {
        let installs = find_pob_dirs();
        let (tx, _rx) = broadcast::channel::<PobBuild>(32);

        let mut keepalive = Vec::new();

        let (raw_tx, mut raw_rx) = mpsc::unbounded_channel::<PathBuf>();

        for install in &installs {
            let raw_tx = raw_tx.clone();
            let dir = install.builds_dir.clone();
            let mut w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(ev) = res {
                    if matches!(
                        ev.kind,
                        EventKind::Create(_) | EventKind::Modify(_)
                    ) {
                        for p in ev.paths {
                            if p.extension().and_then(|s| s.to_str()) == Some("xml") {
                                let _ = raw_tx.send(p);
                            }
                        }
                    }
                }
            })?;
            w.watch(&dir, RecursiveMode::Recursive)?;
            keepalive.push(w);
        }

        let bc = tx.clone();
        tokio::spawn(async move {
            let mut last_path: Option<PathBuf> = None;
            let mut debounce = tokio::time::interval(Duration::from_millis(300));
            debounce.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    Some(path) = raw_rx.recv() => {
                        last_path = Some(path);
                    }
                    _ = debounce.tick() => {
                        if let Some(p) = last_path.take() {
                            tokio::time::sleep(Duration::from_millis(150)).await;
                            match parse_file(&p) {
                                Ok(build) => {
                                    let _ = bc.send(build);
                                }
                                Err(e) => {
                                    tracing::warn!(target: "pob_watcher", "parse fail {}: {:?}", p.display(), e);
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(PobWatcher {
            installs,
            events: tx,
            _keepalive: keepalive,
        })
    }

    pub fn initial_build(&self) -> Option<PobBuild> {
        let mut latest: Option<(PobBuild, std::time::SystemTime)> = None;
        for install in &self.installs {
            if let Some(path) = most_recent_build(&install.builds_dir) {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if let Ok(mtime) = meta.modified() {
                        if let Ok(build) = parse_file(&path) {
                            match &latest {
                                Some((_, t)) if *t >= mtime => {}
                                _ => latest = Some((build, mtime)),
                            }
                        }
                    }
                }
            }
        }
        latest.map(|(b, _)| b)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PobBuild> {
        self.events.subscribe()
    }

    /// Walk every detected install dir, parse a lightweight summary for each
    /// XML file. Sorted by modification time descending so the build picker
    /// shows the most recently saved first.
    pub fn list_all_builds(&self) -> Vec<PobBuildSummary> {
        let mut out: Vec<PobBuildSummary> = Vec::new();
        for install in &self.installs {
            walk_xml(&install.builds_dir, &mut out);
        }
        out.sort_by(|a, b| match (b.mtime, a.mtime) {
            (Some(b), Some(a)) => b.cmp(&a),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        });
        out
    }
}

fn walk_xml(dir: &std::path::Path, out: &mut Vec<PobBuildSummary>) {
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_xml(&path, out);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("xml") {
            continue;
        }
        match parse_summary(&path) {
            Ok(s) => out.push(s),
            Err(e) => tracing::debug!(target: "pob_watcher", "summary skip {}: {}", path.display(), e),
        }
    }
}
