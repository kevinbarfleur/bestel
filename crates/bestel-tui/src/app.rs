use std::io::{stdout, Stdout};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind,
    KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures_util::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::{broadcast, mpsc};

use bestel_core::llm::anthropic::AnthropicClient;
use bestel_core::llm::tools::BuildContext;
use bestel_core::llm::{ChatMessage, LlmDelta, Role};
use bestel_core::pob::watcher::PobWatcher;
use bestel_core::pob::PobBuild;

use crate::ui;

pub enum ChatRole {
    User,
    Assistant,
    System,
}

pub struct ChatLine {
    pub role: ChatRole,
    pub text: String,
}

pub struct AppState {
    pub build: Option<PobBuild>,
    pub history: Vec<ChatMessage>,
    pub display: Vec<ChatLine>,
    pub input: String,
    pub streaming: bool,
    pub model_label: String,
    pub watching_dirs: Vec<String>,
    pub status: String,
    pub scroll: u16,
}

impl AppState {
    fn new(model_label: String, watching_dirs: Vec<String>) -> Self {
        let mut display = Vec::new();
        display.push(ChatLine {
            role: ChatRole::System,
            text: "Bestel observe les vieux récits. Pose ta question, exilé.".into(),
        });
        Self {
            build: None,
            history: Vec::new(),
            display,
            input: String::new(),
            streaming: false,
            model_label,
            watching_dirs,
            status: String::new(),
            scroll: 0,
        }
    }

    pub fn push_assistant_chunk(&mut self, chunk: &str) {
        if let Some(last) = self.display.last_mut() {
            if matches!(last.role, ChatRole::Assistant) {
                last.text.push_str(chunk);
                return;
            }
        }
        self.display.push(ChatLine {
            role: ChatRole::Assistant,
            text: chunk.to_string(),
        });
    }
}

enum AppEvent {
    Crossterm(crossterm::event::Event),
    Pob(PobBuild),
    Llm(LlmDelta),
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

    let client_result = AnthropicClient::from_env();
    let model_label = match &client_result {
        Ok(c) => c.model().to_string(),
        Err(_) => "no API key".into(),
    };
    let client = client_result.ok().map(Arc::new);

    let mut state = AppState::new(model_label, watching_dirs);
    state.build = initial_build;
    if client.is_none() {
        state.status =
            "ANTHROPIC_API_KEY manquante — Bestel ne peut parler. Définis-la et relance."
                .into();
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
        client.clone(),
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
    client: Option<Arc<AnthropicClient>>,
    ctx: BuildContext,
) -> Result<()> {
    terminal.draw(|f| ui::render(f, state))?;

    let mut tick = tokio::time::interval(Duration::from_millis(100));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        let app_event: Option<AppEvent> = tokio::select! {
            maybe = events.next() => {
                match maybe {
                    Some(Ok(ev)) => Some(AppEvent::Crossterm(ev)),
                    Some(Err(_)) | None => return Ok(()),
                }
            }
            res = pob_rx.recv() => {
                match res {
                    Ok(b) => Some(AppEvent::Pob(b)),
                    Err(_) => None,
                }
            }
            Some(d) = llm_rx.recv() => {
                Some(AppEvent::Llm(d))
            }
            _ = tick.tick() => None,
        };

        if let Some(ev) = app_event {
            match ev {
                AppEvent::Crossterm(Event::Key(k)) if k.kind == KeyEventKind::Press => {
                    if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
                        return Ok(());
                    }
                    match k.code {
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Enter => {
                            if !state.streaming {
                                let text = state.input.trim().to_string();
                                if !text.is_empty() {
                                    state.input.clear();
                                    state.display.push(ChatLine {
                                        role: ChatRole::User,
                                        text: text.clone(),
                                    });
                                    state.history.push(ChatMessage {
                                        role: Role::User,
                                        content: text,
                                    });
                                    if let Some(client) = client.clone() {
                                        state.streaming = true;
                                        state.status = "Bestel consulte les vieux récits…".into();
                                        let history = state.history.clone();
                                        let ctx = ctx.clone();
                                        let tx = llm_tx.clone();
                                        tokio::spawn(async move {
                                            if let Err(e) =
                                                client.run(history, ctx, tx.clone()).await
                                            {
                                                let _ = tx.send(LlmDelta::Error(e.to_string()));
                                            }
                                        });
                                    } else {
                                        state.status =
                                            "Pas de clé API : impossible de répondre.".into();
                                    }
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            state.input.pop();
                        }
                        KeyCode::Char(c) => {
                            state.input.push(c);
                        }
                        KeyCode::PageUp => {
                            state.scroll = state.scroll.saturating_add(5);
                        }
                        KeyCode::PageDown => {
                            state.scroll = state.scroll.saturating_sub(5);
                        }
                        _ => {}
                    }
                }
                AppEvent::Crossterm(Event::Resize(_, _)) => {}
                AppEvent::Crossterm(_) => {}
                AppEvent::Pob(b) => {
                    ctx.set(b.clone());
                    state.status = format!("PoB mis à jour : {}", b.summary_line());
                    state.build = Some(b);
                }
                AppEvent::Llm(d) => match d {
                    LlmDelta::Text(t) => {
                        state.push_assistant_chunk(&t);
                    }
                    LlmDelta::ToolCall { name } => {
                        state.status = format!("Bestel invoque : {}", name);
                    }
                    LlmDelta::ToolResult { name } => {
                        state.status = format!("Réponse du parchemin : {}", name);
                    }
                    LlmDelta::End => {
                        state.streaming = false;
                        state.status.clear();
                        if let Some(last) = state.display.last() {
                            if matches!(last.role, ChatRole::Assistant) {
                                state.history.push(ChatMessage {
                                    role: Role::Assistant,
                                    content: last.text.clone(),
                                });
                            }
                        }
                    }
                    LlmDelta::Error(msg) => {
                        state.streaming = false;
                        state.status = format!("Erreur : {}", msg);
                        state.display.push(ChatLine {
                            role: ChatRole::System,
                            text: format!("⚠ {}", msg),
                        });
                    }
                },
            }
        }

        terminal.draw(|f| ui::render(f, state))?;
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(out);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn teardown_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
