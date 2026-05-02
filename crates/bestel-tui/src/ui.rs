use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::AppState;
use crate::items::{fmt_elapsed, ChatItem};
use crate::markdown;
use bestel_core::llm::ToolStatus;
use bestel_core::pob::PobBuild;

const SPINNER: &[&str] = &[
    "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
];

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
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
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
    for (i, item) in state.items.iter().enumerate() {
        let focused = state.focus == Some(i);
        let item_lines = render_item(item, focused, state.spinner_frame, state.streaming);
        lines.extend(item_lines);
        lines.push(Line::from(""));
    }

    let total = lines.len() as u16;
    let visible = chunks[0].height.saturating_sub(2);
    let max_scroll = total.saturating_sub(visible);
    let scroll = if state.follow_tail {
        max_scroll
    } else {
        max_scroll.saturating_sub(state.scroll.min(max_scroll))
    };

    let p = Paragraph::new(lines)
        .block(chat_block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(p, chunks[0]);

    render_input(f, state, chunks[1]);
}

fn render_item(
    item: &ChatItem,
    focused: bool,
    spinner_frame: usize,
    streaming: bool,
) -> Vec<Line<'static>> {
    match item {
        ChatItem::User(text) => render_user(text),
        ChatItem::System(text) => render_system(text),
        ChatItem::Reasoning {
            summary,
            complete,
            expanded,
            ..
        } => render_reasoning(item, summary, *complete, *expanded, focused, spinner_frame),
        ChatItem::Tool {
            name,
            detail,
            output,
            status,
            expanded,
            ..
        } => render_tool(
            item,
            name,
            detail.as_deref(),
            output,
            *status,
            *expanded,
            focused,
            spinner_frame,
        ),
        ChatItem::Assistant {
            text,
            complete,
            ..
        } => render_assistant(text, *complete, streaming, spinner_frame),
    }
}

fn render_user(text: &str) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    let mut first = true;
    for line in text.split('\n') {
        if first {
            out.push(Line::from(vec![
                Span::styled(
                    "you · ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(line.to_string()),
            ]));
            first = false;
        } else {
            out.push(Line::from(vec![
                Span::styled("       ", Style::default()),
                Span::raw(line.to_string()),
            ]));
        }
    }
    out
}

fn render_system(text: &str) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    for line in text.split('\n') {
        out.push(Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::DarkGray),
        )));
    }
    out
}

fn render_reasoning(
    item: &ChatItem,
    summary: &str,
    complete: bool,
    expanded: bool,
    focused: bool,
    spinner_frame: usize,
) -> Vec<Line<'static>> {
    let elapsed = item.elapsed().map(fmt_elapsed).unwrap_or_default();
    let arrow = if expanded { "▾" } else { "▸" };
    let badge = if complete {
        format!("✓ médité · {}", elapsed)
    } else {
        let frame = SPINNER[spinner_frame % SPINNER.len()];
        format!("{} médite · {}", frame, elapsed)
    };
    let header_style = Style::default().fg(if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    }).add_modifier(if focused { Modifier::BOLD } else { Modifier::empty() });

    let mut out = Vec::new();
    out.push(Line::from(vec![
        Span::styled(arrow, header_style),
        Span::raw(" "),
        Span::styled("Bestel ", header_style),
        Span::styled(badge, Style::default().fg(Color::DarkGray)),
        Span::styled(
            if focused { "  [Enter fold]" } else { "" },
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    if expanded {
        for line in summary.split('\n') {
            out.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    line.to_string(),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]));
        }
    } else if !summary.is_empty() {
        let preview = first_line(summary, 80);
        out.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                preview,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
    }
    out
}

fn render_tool(
    item: &ChatItem,
    name: &str,
    detail: Option<&str>,
    output: &str,
    status: ToolStatus,
    expanded: bool,
    focused: bool,
    spinner_frame: usize,
) -> Vec<Line<'static>> {
    let elapsed = item.elapsed().map(fmt_elapsed).unwrap_or_default();
    let (status_label, status_color) = match status {
        ToolStatus::Running => {
            let frame = SPINNER[spinner_frame % SPINNER.len()];
            (format!("{} {}", frame, elapsed), Color::Yellow)
        }
        ToolStatus::Done => (format!("✓ {}", elapsed), Color::Green),
        ToolStatus::Failed => (format!("✗ {}", elapsed), Color::Red),
    };

    let arrow = if expanded { "▾" } else { "▸" };
    let header_color = if focused { Color::Yellow } else { Color::Magenta };
    let header_modifier = if focused { Modifier::BOLD } else { Modifier::empty() };

    let mut header: Vec<Span<'static>> = vec![
        Span::styled(arrow, Style::default().fg(header_color).add_modifier(header_modifier)),
        Span::raw(" "),
        Span::styled(
            "⚙ ",
            Style::default().fg(header_color).add_modifier(header_modifier),
        ),
        Span::styled(
            name.to_string(),
            Style::default().fg(header_color).add_modifier(header_modifier),
        ),
    ];
    if let Some(d) = detail {
        header.push(Span::raw(" · "));
        header.push(Span::styled(
            d.to_string(),
            Style::default().fg(Color::Gray),
        ));
    }
    header.push(Span::raw("  "));
    header.push(Span::styled(
        status_label,
        Style::default().fg(status_color),
    ));
    if focused {
        header.push(Span::styled(
            "  [Enter fold]",
            Style::default().fg(Color::DarkGray),
        ));
    }

    let mut out = Vec::new();
    out.push(Line::from(header));

    if expanded && !output.is_empty() {
        for line in output.split('\n') {
            out.push(Line::from(vec![
                Span::styled("  ↳ ", Style::default().fg(Color::DarkGray)),
                Span::styled(line.to_string(), Style::default().fg(Color::Gray)),
            ]));
        }
    } else if !expanded && !output.is_empty() {
        let preview = first_line(output, 80);
        out.push(Line::from(vec![
            Span::styled("  ↳ ", Style::default().fg(Color::DarkGray)),
            Span::styled(preview, Style::default().fg(Color::Gray)),
        ]));
    }
    out
}

fn render_assistant(
    text: &str,
    complete: bool,
    streaming: bool,
    spinner_frame: usize,
) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();

    let prefix_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    if text.is_empty() && !complete && streaming {
        let frame = SPINNER[spinner_frame % SPINNER.len()];
        out.push(Line::from(vec![
            Span::styled("Bestel · ", prefix_style),
            Span::styled(
                format!("{} ", frame),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "consulte les vieux récits…",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
        return out;
    }

    let body_lines = markdown::render(text);
    for (i, line) in body_lines.into_iter().enumerate() {
        if i == 0 {
            let mut spans: Vec<Span<'static>> = vec![Span::styled("Bestel · ", prefix_style)];
            spans.extend(line.spans);
            out.push(Line::from(spans));
        } else {
            let mut spans: Vec<Span<'static>> = vec![Span::raw("         ")];
            spans.extend(line.spans);
            out.push(Line::from(spans));
        }
    }

    if !complete && streaming {
        let cursor_visible = (spinner_frame / 5) % 2 == 0;
        if cursor_visible {
            if let Some(last) = out.last_mut() {
                last.spans.push(Span::styled(
                    "▌",
                    Style::default().fg(Color::Yellow),
                ));
            } else {
                out.push(Line::from(Span::styled(
                    "▌",
                    Style::default().fg(Color::Yellow),
                )));
            }
        }
    }

    out
}

fn first_line(s: &str, max: usize) -> String {
    let line = s.lines().next().unwrap_or(s).trim();
    if line.chars().count() <= max {
        line.to_string()
    } else {
        let mut out: String = line.chars().take(max).collect();
        out.push('…');
        out
    }
}

fn render_input(f: &mut Frame, state: &AppState, area: Rect) {
    let title = if state.focus.is_some() {
        " > Tab cycle · t fold thinking · o fold tool · Enter toggle "
    } else if state.streaming {
        " > Esc pour annuler "
    } else {
        " > parle, exilé "
    };
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title,
            Style::default().fg(Color::DarkGray),
        ));
    let input_text = if state.streaming {
        Line::from(Span::styled(
            "(Bestel répond — Esc pour annuler)",
            Style::default().fg(Color::DarkGray),
        ))
    } else if state.focus.is_some() {
        Line::from(Span::styled(
            "(focus actif sur un bloc — Tab pour cycler, Enter pour plier/déplier)",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(Span::raw(state.input.clone()))
    };
    let input = Paragraph::new(input_text).block(input_block);
    f.render_widget(input, area);
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
        " {} · {} ({}) · Ctrl+C quitter ",
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
