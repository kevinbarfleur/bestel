use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{AppState, ChatLine, ChatRole};
use bestel_core::pob::PobBuild;

pub fn render(f: &mut Frame, state: &AppState) {
    let area = f.area();

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let main_area = outer[0];
    let footer_area = outer[1];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
        .split(main_area);

    render_build_panel(f, state.build.as_ref(), chunks[0]);
    render_chat_panel(f, state, chunks[1]);
    render_footer(f, state, footer_area);
}

fn render_build_panel(f: &mut Frame, build: Option<&PobBuild>, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " ⚔  Build ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let lines: Vec<Line> = match build {
        None => vec![
            Line::from(""),
            Line::from(Span::styled(
                "Aucun build chargé.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from("Sauvegarde un build dans"),
            Line::from("Path of Building et il"),
            Line::from("apparaîtra ici."),
        ],
        Some(b) => build_lines(b),
    };

    let p = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn build_lines(b: &PobBuild) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();
    out.push(Line::from(vec![
        Span::styled(
            format!("[{}] ", b.game.label()),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            b.class.clone(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));
    if let Some(asc) = b.ascendancy.as_deref().filter(|s| !s.is_empty()) {
        out.push(Line::from(Span::styled(
            asc.to_string(),
            Style::default().fg(Color::Magenta),
        )));
    }
    if let Some(lvl) = b.level {
        out.push(Line::from(format!("Niveau {}", lvl)));
    }
    if let Some(skill) = &b.main_skill {
        out.push(Line::from(""));
        out.push(Line::from(Span::styled(
            "Compétence",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        out.push(Line::from(skill.clone()));
    }

    out.push(Line::from(""));
    out.push(Line::from(Span::styled(
        "Vie",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    out.push(Line::from(format!(
        "Life {}  Mana {}",
        fmt_int(b.life()),
        fmt_int(b.mana())
    )));
    out.push(Line::from(format!(
        "ES {}  EHP {}",
        fmt_int(b.energy_shield()),
        fmt_int(b.ehp())
    )));

    out.push(Line::from(""));
    out.push(Line::from(Span::styled(
        "DPS",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    out.push(Line::from(fmt_dps(b.dps())));

    out.push(Line::from(""));
    out.push(Line::from(Span::styled(
        "Résistances",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    for (label, val) in b.resistances() {
        let style = match val {
            Some(v) if v >= 75.0 => Style::default().fg(Color::Green),
            Some(v) if v >= 60.0 => Style::default().fg(Color::Yellow),
            Some(_) => Style::default().fg(Color::Red),
            None => Style::default().fg(Color::DarkGray),
        };
        out.push(Line::from(Span::styled(
            format!(
                "{:<10} {}",
                label,
                val.map(|v| format!("{:>3}%", v as i64))
                    .unwrap_or_else(|| "  —".into())
            ),
            style,
        )));
    }

    if !b.items.is_empty() {
        out.push(Line::from(""));
        out.push(Line::from(Span::styled(
            "Équipement",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        for it in b.items.iter().take(8) {
            let label = match (it.name.as_deref(), it.base.as_deref()) {
                (Some(n), Some(b)) if n != b => format!("{} — {}", n, b),
                (Some(n), _) => n.to_string(),
                (None, Some(b)) => b.to_string(),
                _ => format!("Item #{}", it.id),
            };
            let color = match it.rarity.as_deref() {
                Some(r) if r.eq_ignore_ascii_case("UNIQUE") => Color::Yellow,
                Some(r) if r.eq_ignore_ascii_case("RARE") => Color::LightYellow,
                Some(r) if r.eq_ignore_ascii_case("MAGIC") => Color::LightBlue,
                _ => Color::Gray,
            };
            out.push(Line::from(Span::styled(label, Style::default().fg(color))));
        }
    }

    out
}

fn fmt_int(v: Option<f64>) -> String {
    match v {
        Some(v) => format!("{}", v as i64),
        None => "—".into(),
    }
}

fn fmt_dps(v: Option<f64>) -> String {
    match v {
        None => "—".into(),
        Some(v) if v >= 1_000_000.0 => format!("{:.2}M", v / 1_000_000.0),
        Some(v) if v >= 1_000.0 => format!("{:.1}k", v / 1_000.0),
        Some(v) => format!("{:.0}", v),
    }
}

fn render_chat_panel(f: &mut Frame, state: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let chat_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " 📜  Bestel ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let mut lines: Vec<Line> = Vec::new();
    for line in &state.display {
        push_chat(&mut lines, line);
        lines.push(Line::from(""));
    }
    if state.streaming {
        lines.push(Line::from(Span::styled(
            "▌",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let total = lines.len() as u16;
    let visible = chunks[0].height.saturating_sub(2);
    let max_scroll = total.saturating_sub(visible);
    let scroll = max_scroll.saturating_sub(state.scroll.min(max_scroll));

    let p = Paragraph::new(lines)
        .block(chat_block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(p, chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " > parle, exilé ",
            Style::default().fg(Color::DarkGray),
        ));
    let input_text = if state.streaming {
        Line::from(Span::styled(
            "(Bestel répond…)",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(Span::raw(state.input.clone()))
    };
    let input = Paragraph::new(input_text).block(input_block);
    f.render_widget(input, chunks[1]);
}

fn push_chat(lines: &mut Vec<Line<'static>>, msg: &ChatLine) {
    let (prefix, style) = match msg.role {
        ChatRole::User => (
            "you · ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        ChatRole::Assistant => (
            "Bestel · ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        ChatRole::System => ("· ", Style::default().fg(Color::DarkGray)),
    };
    let mut first = true;
    for line in msg.text.split('\n') {
        if first {
            lines.push(Line::from(vec![
                Span::styled(prefix, style),
                Span::raw(line.to_string()),
            ]));
            first = false;
        } else {
            lines.push(Line::from(Span::raw(line.to_string())));
        }
    }
}

fn render_footer(f: &mut Frame, state: &AppState, area: Rect) {
    let pob_status = if state.build.is_some() {
        "PoB ✓"
    } else if state.watching_dirs.is_empty() {
        "PoB ✗"
    } else {
        "PoB ⏳"
    };
    let left = format!(
        " {} · {} ({}) · Ctrl+C pour quitter ",
        pob_status, state.model_label, state.auth_label
    );
    let right = if state.status.is_empty() {
        String::new()
    } else {
        format!(" {} ", state.status)
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let l = Paragraph::new(Line::from(Span::styled(
        left,
        Style::default().fg(Color::DarkGray),
    )));
    let r = Paragraph::new(Line::from(Span::styled(
        right,
        Style::default().fg(Color::DarkGray),
    )))
    .alignment(ratatui::layout::Alignment::Right);
    f.render_widget(l, chunks[0]);
    f.render_widget(r, chunks[1]);
}
