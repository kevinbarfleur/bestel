use std::collections::HashMap;
use std::io::{stdout, Stdout, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
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
    pub focus: Option<usize>,
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
            focus: None,
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
                    if let Some(ChatItem::Reasoning { summary, .. }) =
                        self.items.get_mut(idx)
                    {
                        summary.push_str(&s);
                    }
                }
            }
            LlmDelta::ReasoningEnd => {
                if let Some(idx) = self.current_reasoning_idx.take() {
                    if let Some(ChatItem::Reasoning {
                        complete,
                        expanded,
                        ended,
                        ..
                    }) = self.items.get_mut(idx)
                    {
                        *complete = true;
                        *expanded = false;
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
                        output,
                        ..
                    }) = self.items.get_mut(idx)
                    {
                        *s = status;
                        *ended = Some(Instant::now());
                        if let Some(sum) = summary {
                            if !sum.is_empty() && !output.contains(&sum) {
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

    fn cycle_focus(&mut self, forward: bool) {
        let expandable: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, it)| it.is_expandable())
            .map(|(i, _)| i)
            .collect();
        if expandable.is_empty() {
            self.focus = None;
            return;
        }
        let pos = match self.focus {
            None => {
                if forward {
                    Some(0)
                } else {
                    Some(expandable.len() - 1)
                }
            }
            Some(curr) => expandable.iter().position(|&i| i == curr).map(|p| {
                if forward {
                    (p + 1) % expandable.len()
                } else {
                    if p == 0 {
                        expandable.len() - 1
                    } else {
                        p - 1
                    }
                }
            }),
        };
        self.focus = pos.map(|p| expandable[p]);
    }

    fn toggle_focus(&mut self) {
        if let Some(idx) = self.focus {
            if let Some(item) = self.items.get_mut(idx) {
                item.toggle_expanded();
            }
        }
    }

    fn toggle_last_reasoning(&mut self) {
        for item in self.items.iter_mut().rev() {
            if matches!(item, ChatItem::Reasoning { .. }) {
                item.toggle_expanded();
                return;
            }
        }
    }

    fn toggle_focus_or<F: Fn(&ChatItem) -> bool>(&mut self, predicate: F) {
        if let Some(idx) = self.focus {
            if let Some(it) = self.items.get(idx) {
                if predicate(it) {
                    if let Some(it) = self.items.get_mut(idx) {
                        it.toggle_expanded();
                        return;
                    }
                }
            }
        }
        for item in self.items.iter_mut().rev() {
            if predicate(item) {
                item.toggle_expanded();
                return;
            }
        }
    }
}

enum AppEvent {
    Crossterm(Event),
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

    let mut terminal = setup_terminal()?;
    let mut events = EventStream::new();
    let (llm_tx, mut llm_rx) = mpsc::unbounded_channel::<LlmDelta>();

    let result = run_loop(
        &mut terminal,
        &mut state,
        &mut events,
        &mut pob_rx,
        &mut llm_rx,
        &llm_tx,
        provider.clone(),
        ctx.clone(),
    )
    .await;

    teardown_terminal(&mut terminal)?;
    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut AppState,
    events: &mut EventStream,
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
            maybe = events.next() => {
                match maybe {
                    Some(Ok(ev)) => AppEvent::Crossterm(ev),
                    Some(Err(_)) | None => return Ok(()),
                }
            }
            res = pob_rx.recv() => {
                match res {
                    Ok(b) => AppEvent::Pob(b),
                    Err(_) => continue,
                }
            }
            Some(d) = llm_rx.recv() => AppEvent::Llm(d),
            _ = tick.tick() => AppEvent::Tick,
        };

        let mut needs_redraw = true;
        let is_tick = matches!(app_event, AppEvent::Tick);

        match app_event {
            AppEvent::Crossterm(Event::Key(k)) if k.kind == KeyEventKind::Press => {
                if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
                    return Ok(());
                }
                match k.code {
                    KeyCode::Esc => {
                        if state.streaming {
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
                            for (id, idx) in std::mem::take(&mut state.current_tool_idx) {
                                if let Some(ChatItem::Tool {
                                    status, ended, ..
                                }) = state.items.get_mut(idx)
                                {
                                    *status = ToolStatus::Failed;
                                    *ended = Some(Instant::now());
                                }
                                let _ = id;
                            }
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Enter => {
                        if state.focus.is_some() {
                            state.toggle_focus();
                        } else if !state.streaming {
                            let text = state.input.trim().to_string();
                            if !text.is_empty() {
                                state.input.clear();
                                devlog::log_value(
                                    "user_input",
                                    json!({ "text": text }),
                                );
                                state.push_item(ChatItem::User(text.clone()));
                                state.history.push(ChatMessage {
                                    role: Role::User,
                                    content: text,
                                });
                                state.follow_tail = true;
                                if let Some(p) = provider.clone() {
                                    state.streaming = true;
                                    state.status = "Bestel consulte les vieux récits…".into();
                                    let history = state.history.clone();
                                    let ctx = ctx.clone();
                                    let tx = llm_tx.clone();
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
                                                // Provider task dropped, kill_on_drop on subprocess takes care of cleanup.
                                            }
                                        }
                                    });
                                } else {
                                    state.status = "Aucun provider disponible.".into();
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if !state.streaming && state.focus.is_none() {
                            state.input.pop();
                        }
                    }
                    KeyCode::Tab => {
                        state.cycle_focus(true);
                    }
                    KeyCode::BackTab => {
                        state.cycle_focus(false);
                    }
                    KeyCode::Char('t') if state.streaming || state.focus.is_some() => {
                        state.toggle_focus_or(|it| matches!(it, ChatItem::Reasoning { .. }));
                    }
                    KeyCode::Char('o') if state.streaming || state.focus.is_some() => {
                        state.toggle_focus_or(|it| matches!(it, ChatItem::Tool { .. }));
                    }
                    KeyCode::Char(c) => {
                        if state.focus.is_none() && !state.streaming {
                            state.input.push(c);
                        } else if state.focus.is_some() {
                            // Letters while focused: t = thinking, o = tool, leave others as no-op
                            match c {
                                't' => state
                                    .toggle_focus_or(|it| matches!(it, ChatItem::Reasoning { .. })),
                                'o' => state
                                    .toggle_focus_or(|it| matches!(it, ChatItem::Tool { .. })),
                                'g' => {
                                    state.scroll = u16::MAX;
                                    state.follow_tail = false;
                                }
                                'G' => {
                                    state.scroll = 0;
                                    state.follow_tail = true;
                                }
                                'j' => state.scroll = state.scroll.saturating_sub(2),
                                'k' => {
                                    state.scroll = state.scroll.saturating_add(2);
                                    state.follow_tail = false;
                                }
                                _ => {}
                            }
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
            }
            AppEvent::Crossterm(Event::Resize(_, _)) => {}
            AppEvent::Crossterm(_) => {
                needs_redraw = false;
            }
            AppEvent::Pob(b) => {
                ctx.set(b.clone());
                state.status = format!("PoB mis à jour : {}", b.summary_line());
                state.build = Some(b);
            }
            AppEvent::Llm(d) => {
                state.handle_delta(d);
            }
            AppEvent::Tick => {
                if state.streaming || has_running_item(state) {
                    state.spinner_frame = state.spinner_frame.wrapping_add(1);
                } else {
                    needs_redraw = false;
                }
            }
        }

        if needs_redraw {
            // Throttle ticks to ~30 fps so the terminal has idle gaps for
            // mouse selection. Non-tick events always render so the user
            // sees a response to their actions and to LLM deltas.
            let throttle_ok = state
                .last_render_at
                .map(|t| t.elapsed() >= Duration::from_millis(33))
                .unwrap_or(true);
            let must = !is_tick || throttle_ok;
            if must {
                draw(terminal, state)?;
                state.last_render_at = Some(Instant::now());
            }
        }
    }
}

fn log_delta(d: &LlmDelta) {
    if !devlog::is_enabled() {
        return;
    }
    let (kind, payload) = match d {
        LlmDelta::TextDelta(t) => (
            "text_delta",
            json!({ "len": t.chars().count(), "text": t }),
        ),
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
        LlmDelta::ToolOutput { id, chunk } => (
            "tool_output",
            json!({ "id": id, "chunk": chunk }),
        ),
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

fn draw(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &AppState,
) -> Result<()> {
    // Synchronized output (BSU/ESU mode 2026) eliminates flicker on
    // supported terminals but can confuse mouse handling on some others
    // when emitted at high frequency. Opt-in via BESTEL_SYNC_OUTPUT=1.
    let sync = std::env::var("BESTEL_SYNC_OUTPUT")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false);
    if sync {
        let mut out = stdout();
        let _ = out.write_all(b"\x1b[?2026h");
        let _ = out.flush();
    }
    terminal.draw(|f| ui::render(f, state))?;
    if sync {
        let mut out = stdout();
        let _ = out.write_all(b"\x1b[?2026l");
        let _ = out.flush();
    }
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn teardown_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
