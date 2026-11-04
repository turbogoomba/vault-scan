use std::fs;
use std::path::Path;

use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};

use crate::theme::{ACCENT, MUTED, TEXT};

// Reads a note and turns the markdown into styled lines
pub fn render(path: &Path) -> Vec<Line<'static>> {
    let Ok(text) = fs::read_to_string(path) else {
        return vec![Line::from("Kan ikke lese filen").fg(MUTED)];
    };

    let mut lines = Vec::new();
    let mut in_code = false;
    let mut in_front = false;

    for (i, raw) in text.lines().enumerate() {
        // Skip the frontmatter between the --- lines at the top
        if i == 0 && raw.trim() == "---" {
            in_front = true;
            continue;
        }
        if in_front {
            if raw.trim() == "---" {
                in_front = false;
            }
            continue;
        }

        let t = raw.trim_start();

        // Code blocks are shown dimmed, as they are
        if t.starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            lines.push(Line::from(format!("  {}", raw)).fg(MUTED));
            continue;
        }

        let line = if let Some(h) = t.strip_prefix("# ") {
            Line::from(h.to_string()).fg(TEXT).bold()
        } else if let Some(h) = t.strip_prefix("## ") {
            Line::from(h.to_string()).fg(ACCENT).bold()
        } else if let Some(h) = t.strip_prefix("### ") {
            Line::from(h.to_string()).fg(ACCENT)
        } else if let Some(item) = t.strip_prefix("- ").or(t.strip_prefix("* ")) {
            let indent = " ".repeat(raw.len() - t.len());
            let mut spans = vec![Span::raw(format!("{}  • ", indent)).fg(ACCENT)];
            spans.extend(inline(item, Style::new().fg(TEXT)));
            Line::from(spans)
        } else if let Some(q) = t.strip_prefix("> ") {
            let mut spans = vec![Span::raw("│ ").fg(MUTED)];
            spans.extend(inline(q, Style::new().fg(MUTED).italic()));
            Line::from(spans)
        } else if t.starts_with("$$") {
            Line::from(raw.to_string()).fg(ACCENT).italic()
        } else {
            Line::from(inline(raw, Style::new().fg(TEXT)))
        };

        lines.push(line);
    }
    lines
}

// Handles **bold** and [[links]] inside a line
fn inline(text: &str, base: Style) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut bold = false;
    let mut rest = text;

    while !rest.is_empty() {
        let next_bold = rest.find("**");
        let next_link = rest.find("[[");

        match (next_bold, next_link) {
            // ** comes first: switch bold on or off
            (Some(b), l) if l.map_or(true, |l| b < l) => {
                push(&mut spans, &rest[..b], base, bold);
                bold = !bold;
                rest = &rest[b + 2..];
            }
            // [[ comes first: show the link name underlined
            (_, Some(l)) => {
                push(&mut spans, &rest[..l], base, bold);
                let after = &rest[l + 2..];
                if let Some(end) = after.find("]]") {
                    let link = &after[..end];
                    let shown = link.split('|').last().unwrap_or(link);
                    spans.push(Span::styled(shown.to_string(), Style::new().fg(ACCENT).underlined()));
                    rest = &after[end + 2..];
                } else {
                    push(&mut spans, "[[", base, bold);
                    rest = after;
                }
            }
            // Nothing special left
            _ => {
                push(&mut spans, rest, base, bold);
                break;
            }
        }
    }
    spans
}

// Adds a piece of text, bold or not
fn push(spans: &mut Vec<Span<'static>>, text: &str, base: Style, bold: bool) {
    if !text.is_empty() {
        let style = if bold { base.bold() } else { base };
        spans.push(Span::styled(text.to_string(), style));
    }
}
