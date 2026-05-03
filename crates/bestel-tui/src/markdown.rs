use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub fn render(text: &str) -> Vec<Line<'static>> {
    let normalized = normalize_indent(text);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut in_code_block = false;
    let mut code_block_lang: Option<String> = None;
    let mut current_code: Vec<String> = Vec::new();

    for raw in normalized.split('\n') {
        let line = raw;
        let trimmed = line.trim_start();

        if let Some(rest) = trimmed.strip_prefix("```") {
            if in_code_block {
                for code_line in current_code.drain(..) {
                    lines.push(code_block_line(code_line));
                }
                in_code_block = false;
                code_block_lang = None;
            } else {
                let lang = rest.trim().to_string();
                code_block_lang = if lang.is_empty() { None } else { Some(lang) };
                in_code_block = true;
                let label = match &code_block_lang {
                    Some(l) => format!("┌─ {} ─", l),
                    None => "┌──".to_string(),
                };
                lines.push(Line::from(Span::styled(
                    label,
                    Style::default().fg(Color::DarkGray),
                )));
            }
            continue;
        }

        if in_code_block {
            current_code.push(line.to_string());
            continue;
        }

        // List markers
        if let Some(rest) = strip_list_marker(line) {
            let mut spans = vec![
                Span::styled(rest.0, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
            ];
            spans.extend(inline_spans(&rest.1));
            lines.push(Line::from(spans));
            continue;
        }

        // Headings
        if let Some((level, rest)) = strip_heading(line) {
            let style = match level {
                1 => Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                2 => Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                _ => Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            };
            lines.push(Line::from(Span::styled(rest.to_string(), style)));
            continue;
        }

        // Blockquote
        if let Some(rest) = line.strip_prefix("> ") {
            let mut spans = vec![Span::styled(
                "│ ".to_string(),
                Style::default().fg(Color::DarkGray),
            )];
            spans.extend(inline_spans(rest));
            lines.push(Line::from(spans));
            continue;
        }

        if line.is_empty() {
            lines.push(Line::from(""));
        } else {
            lines.push(Line::from(inline_spans(line)));
        }
    }

    if in_code_block {
        for code_line in current_code.drain(..) {
            lines.push(code_block_line(code_line));
        }
    }

    lines
}

fn code_block_line(content: String) -> Line<'static> {
    Line::from(vec![
        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            content,
            Style::default().fg(Color::LightGreen).bg(Color::Reset),
        ),
    ])
}

fn strip_list_marker(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start();
    let indent_len = line.len() - trimmed.len();
    let indent = " ".repeat(indent_len);

    if let Some(rest) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
        return Some((format!("{}•", indent), rest.to_string()));
    }
    // Numeric "1. "
    let mut chars = trimmed.char_indices();
    let mut digit_end = 0;
    while let Some((i, c)) = chars.next() {
        if c.is_ascii_digit() {
            digit_end = i + 1;
        } else {
            break;
        }
    }
    if digit_end > 0 && digit_end < trimmed.len() {
        let after = &trimmed[digit_end..];
        if let Some(rest) = after.strip_prefix(". ") {
            let marker = &trimmed[..digit_end + 1];
            return Some((format!("{}{}", indent, marker), rest.to_string()));
        }
    }
    None
}

fn strip_heading(line: &str) -> Option<(usize, &str)> {
    let mut count = 0;
    let bytes = line.as_bytes();
    while count < bytes.len() && bytes[count] == b'#' {
        count += 1;
    }
    if count == 0 || count > 6 || count >= bytes.len() {
        return None;
    }
    if bytes[count] != b' ' {
        return None;
    }
    Some((count, &line[count + 1..]))
}

pub fn inline_spans(text: &str) -> Vec<Span<'static>> {
    let mut out: Vec<Span<'static>> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut buf = String::new();

    let flush = |buf: &mut String, out: &mut Vec<Span<'static>>| {
        if !buf.is_empty() {
            out.push(Span::raw(std::mem::take(buf)));
        }
    };

    while i < chars.len() {
        // Markdown link [label](url)
        if chars[i] == '[' {
            if let Some((label, url, end)) = parse_link(&chars, i) {
                let mut tmp = String::new();
                std::mem::swap(&mut tmp, &mut buf);
                if !tmp.is_empty() {
                    out.push(Span::raw(tmp));
                }
                out.extend(make_hyperlink(&label, &url));
                i = end;
                continue;
            }
        }
        // Inline code `code`
        if chars[i] == '`' {
            if let Some((code, end)) = parse_until(&chars, i + 1, '`') {
                let mut tmp = String::new();
                std::mem::swap(&mut tmp, &mut buf);
                if !tmp.is_empty() {
                    out.push(Span::raw(tmp));
                }
                out.push(Span::styled(
                    code,
                    Style::default().fg(Color::LightGreen).bg(Color::Black),
                ));
                i = end + 1;
                continue;
            }
        }
        // Bold **text**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some((bold, end)) = parse_double_marker(&chars, i + 2, '*') {
                let mut tmp = String::new();
                std::mem::swap(&mut tmp, &mut buf);
                if !tmp.is_empty() {
                    out.push(Span::raw(tmp));
                }
                let inner = inline_spans(&bold);
                for span in inner {
                    let style = span.style.add_modifier(Modifier::BOLD);
                    out.push(Span::styled(span.content.into_owned(), style));
                }
                i = end + 2;
                continue;
            }
        }
        // Italic *text*
        if chars[i] == '*' {
            if let Some((italic, end)) = parse_until(&chars, i + 1, '*') {
                let mut tmp = String::new();
                std::mem::swap(&mut tmp, &mut buf);
                if !tmp.is_empty() {
                    out.push(Span::raw(tmp));
                }
                out.push(Span::styled(
                    italic,
                    Style::default().add_modifier(Modifier::ITALIC),
                ));
                i = end + 1;
                continue;
            }
        }

        buf.push(chars[i]);
        i += 1;
    }

    flush(&mut buf, &mut out);
    out
}

fn parse_link(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    if start >= chars.len() || chars[start] != '[' {
        return None;
    }
    let mut i = start + 1;
    let mut label = String::new();
    while i < chars.len() && chars[i] != ']' {
        label.push(chars[i]);
        i += 1;
    }
    if i >= chars.len() || i + 1 >= chars.len() || chars[i + 1] != '(' {
        return None;
    }
    i += 2;
    let mut url = String::new();
    while i < chars.len() && chars[i] != ')' {
        url.push(chars[i]);
        i += 1;
    }
    if i >= chars.len() {
        return None;
    }
    Some((label, url, i + 1))
}

fn parse_until(chars: &[char], start: usize, marker: char) -> Option<(String, usize)> {
    let mut i = start;
    let mut buf = String::new();
    while i < chars.len() {
        if chars[i] == marker {
            return Some((buf, i));
        }
        buf.push(chars[i]);
        i += 1;
    }
    None
}

fn parse_double_marker(chars: &[char], start: usize, marker: char) -> Option<(String, usize)> {
    let mut i = start;
    let mut buf = String::new();
    while i + 1 < chars.len() {
        if chars[i] == marker && chars[i + 1] == marker {
            return Some((buf, i));
        }
        buf.push(chars[i]);
        i += 1;
    }
    None
}

fn make_hyperlink(label: &str, url: &str) -> Vec<Span<'static>> {
    // Plain text URLs work better than OSC 8 inside ratatui Paragraph:
    // OSC 8 escape bytes confuse the wrap algorithm and corrupt nearby text.
    // Most modern terminals (Windows Terminal, iTerm2, Ghostty, WezTerm)
    // auto-detect plain URLs and make them clickable (Ctrl+Click on Windows).
    vec![
        Span::styled(
            label.to_string(),
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::UNDERLINED),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            url.to_string(),
            Style::default().fg(Color::Blue),
        ),
    ]
}

/// Strip the common minimum indent from every non-empty line.
/// Many models (Codex/GPT-5 in particular) over-indent their output;
/// this aligns paragraphs back to the left without hurting code blocks
/// (which keep their internal relative indentation).
fn normalize_indent(text: &str) -> String {
    let mut min_indent: Option<usize> = None;
    let mut in_code_block = false;
    for raw in text.split('\n') {
        if raw.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if raw.trim().is_empty() {
            continue;
        }
        let indent = raw.chars().take_while(|c| *c == ' ').count();
        min_indent = Some(min_indent.map_or(indent, |m| m.min(indent)));
    }
    let strip = min_indent.unwrap_or(0).min(8);
    if strip == 0 {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut in_code_block = false;
    for raw in text.split('\n') {
        if raw.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
        }
        if in_code_block {
            out.push_str(raw);
            out.push('\n');
            continue;
        }
        let leading: usize = raw.chars().take_while(|c| *c == ' ').count();
        let take = strip.min(leading);
        out.push_str(&raw[take..]);
        out.push('\n');
    }
    if !text.ends_with('\n') {
        out.pop();
    }
    out
}
