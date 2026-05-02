use std::time::{Duration, Instant};

use bestel_core::llm::ToolStatus;

#[derive(Debug, Clone)]
pub enum ChatItem {
    User(String),
    Reasoning {
        summary: String,
        complete: bool,
        expanded: bool,
        started: Instant,
        ended: Option<Instant>,
    },
    Tool {
        id: String,
        name: String,
        detail: Option<String>,
        output: String,
        status: ToolStatus,
        expanded: bool,
        started: Instant,
        ended: Option<Instant>,
    },
    Assistant {
        text: String,
        complete: bool,
        started: Instant,
    },
    System(String),
}

impl ChatItem {
    pub fn is_expandable(&self) -> bool {
        matches!(self, ChatItem::Reasoning { .. } | ChatItem::Tool { .. })
    }

    pub fn is_expanded(&self) -> bool {
        match self {
            ChatItem::Reasoning { expanded, .. } => *expanded,
            ChatItem::Tool { expanded, .. } => *expanded,
            _ => false,
        }
    }

    pub fn toggle_expanded(&mut self) {
        match self {
            ChatItem::Reasoning { expanded, .. } => *expanded = !*expanded,
            ChatItem::Tool { expanded, .. } => *expanded = !*expanded,
            _ => {}
        }
    }

    pub fn elapsed(&self) -> Option<Duration> {
        match self {
            ChatItem::Reasoning {
                started, ended, ..
            } => Some(ended.unwrap_or_else(Instant::now).saturating_duration_since(*started)),
            ChatItem::Tool {
                started, ended, ..
            } => Some(ended.unwrap_or_else(Instant::now).saturating_duration_since(*started)),
            ChatItem::Assistant { started, .. } => {
                Some(Instant::now().saturating_duration_since(*started))
            }
            _ => None,
        }
    }
}

pub fn fmt_elapsed(d: Duration) -> String {
    let secs = d.as_secs_f32();
    if secs < 60.0 {
        format!("{:.1}s", secs)
    } else {
        let s = d.as_secs();
        format!("{}m{:02}s", s / 60, s % 60)
    }
}
