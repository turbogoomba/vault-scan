use std::path::{PathBuf, Path};
use std::time::SystemTime;

use ratatui::Frame;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine};
use ratatui::layout::{Constraint, Direction, Flex, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{BarChart, Block, BorderType, Clear, Wrap, Paragraph};

use crate::tree::Item;
use crate::preview;
use crate::exams;
use crate::scan::Stats;
use crate::theme::*;

// Draws the dashborad
pub fn draw(frame: &mut Frame, stats: &Stats, p: f32, e: f32, ms: u32,
    selected: usize, searching: bool, query: &str, results: &[(String, PathBuf)],
    tree: &[Item], prompt: Option<&str>) {
    let days = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() / 86400;
    let today = ((days + 3) % 7) as i64;

    let [top, bottom, help_area] = Layout::vertical([
        Constraint::Length(10),
        Constraint::Min(0),
        Constraint::Length(1),
    ]).areas(frame.area());

    let [logo_area] = Layout::horizontal([Constraint::Length(110)])
        .flex(Flex::Center)
        .areas(top);

    // Gradient logo with twinkling stars
    let rows: Vec<Line> = LOGO.lines().enumerate().map(|(i, line)| {
        let t = i as f32 / 9.0;
        let (r, g, b) = (mix(0x51, 0xE3, t), mix(0x51, 0xDC, t), mix(0x3D, 0x95, t));

        let cells: Vec<Span> = line.chars().enumerate().map(|(j, ch)| {
            let (i, j) = (i as u32, j as u32);

            // Each character gets its own rhythm
            let offset = hash(i, j, 0) % 1000;
            let cycle = (ms + offset) / 1000;
            let phase = ((ms + offset) % 1000) as f32 / 1000.0;

            // About 1 in 60 characters lights up each cycle
            let star = p >= 1.0 && ch != ' ' && hash(i, j, cycle) % 60 == 0;
            let glow = if star { (phase * std::f32::consts::PI).sin() } else { 0.0 };

            let color = Color::Rgb(
                mix(mix(0x1B, r, e), 0xFF, glow),
                mix(mix(0x20, g, e), 0xFA, glow),
                mix(mix(0x21, b, e), 0xE0, glow),
            );
            let shown = if glow > 0.7 { '✦' } else { ch };
            Span::raw(shown.to_string()).fg(color)
        }).collect();

        Line::from(cells)
    }).collect();

    frame.render_widget(Block::new().bg(BG), top);
    frame.render_widget(Paragraph::new(rows).bg(BG), logo_area);
    // Heatmap at the bottom, the rest above it
    let [middle, heat_area] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(9),
    ]).areas(bottom);

    // Chart and exams on the left, note list on the right
    let [left, list_area] = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ]).areas(middle);

    // Note list on top, graph below
    let [list_area, graph_area] = Layout::vertical([
        if tree.is_empty() { Constraint::Percentage(50) } else { Constraint::Min(0) },
        if tree.is_empty() { Constraint::Percentage(50) } else { Constraint::Length(0) },
    ]).areas(list_area);

    // Bar chart on top, exams below, half each
    let [chart_area, exam_area] = Layout::vertical([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ]).areas(left);
    // Bar chart
    let bars: Vec<(&str, u64)> = stats.dirs.iter()
        .map(|(n, c)| (n.as_str(), (*c as f32 * e) as u64))
        .collect();
    let max = stats.dirs.iter().map(|(_, c)| *c).max().unwrap_or(1); // NY

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(format!(" vault-scan   {} words   {} notater ", (stats.total as f32 * e) as u32, (stats.notes as f32 * e) as u32))
        .fg(MUTED)
        .bg(BG);
    let chart = BarChart::default()
        .block(block)
        .data(&bars)
        .max(max) // NY
        .direction(Direction::Horizontal)
        .bar_gap(1)
        .bar_style(Style::new().fg(ACCENT))
        .value_style(Style::new().fg(BG).bg(ACCENT))
        .label_style(Style::new().fg(TEXT));
    frame.render_widget(chart, chart_area);
    // Exams, soonest first, with a bar that fills up the last 30 days
    // Exams, soonest first, with a bar that fills up the last 30 days
    let exam_lines: Vec<Line> = exams::upcoming().into_iter().map(|(name, date, left)| {
        let color = if left <= 7 { TEXT } else { ACCENT };
        let when = match left {
            0 => "i dag".to_string(),
            1 => "i morgen".to_string(),
            d => format!("om {} dager", d),
        };

        // How many of the 10 boxes are filled, growing in at startup
        let filled = ((30 - left.min(30)) as usize / 3).min(10);
        let shown = (filled as f32 * e) as usize;

        // Goes smoothly from 0 to 1 and back, once a second
        let pulse = ((ms as f32 / 1000.0 * std::f32::consts::TAU).sin() + 1.0) / 2.0;
        let Color::Rgb(r, g, b) = color else { unreachable!() };
        let glow = Color::Rgb(mix(0x51, r, pulse), mix(0x51, g, pulse), mix(0x3D, b, pulse));

        let mut spans = vec![
            Span::raw(format!(" {:<28}", name)).fg(TEXT),
            Span::raw(format!("{}   ", date)).fg(MUTED),
        ];

        // Filled boxes, then the pulsing next box, then empty boxes
        for i in 0..10 {
            if i < shown {
                spans.push(Span::raw("▰").fg(color));
            } else if i == shown && p >= 1.0 {
                spans.push(Span::raw("▰").fg(glow));
            } else {
                spans.push(Span::raw("▱").fg(MUTED));
            }
        }

        spans.push(Span::raw(format!("   {}", when)).fg(color));
        Line::from(spans)
    }).collect();

    let exam_box = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(" eksamener ")
        .fg(MUTED)
        .bg(BG);
    frame.render_widget(Paragraph::new(exam_lines).block(exam_box), exam_area);
    // The note under the cursor, if we are browsing or searching
    let preview_path: Option<&Path> = if !tree.is_empty() {
        tree.get(selected).filter(|item| !item.is_dir).map(|item| item.path.as_path())
    } else if searching {
        results.get(selected).map(|(_, path)| path.as_path())
    } else {
        None
    };

    // Draw the preview on top of the chart and exams
    if let Some(path) = preview_path {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let preview_box = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!(" {} ", name))
            .fg(MUTED)
            .bg(BG);

        frame.render_widget(Clear, left);
        frame.render_widget(
            Paragraph::new(preview::render(path)).block(preview_box).wrap(Wrap { trim: false }),
            left,
        );
    }

    // Note list: search results, file tree or biggest notes
    let (title, lines): (String, Vec<Line>) = if let Some(text) = prompt {
        (text.to_string(), Vec::new())
    } else if !tree.is_empty() {
        let lines = tree.iter().map(|item| {
            let indent = "  ".repeat(item.depth);
            if item.is_dir {
                let arrow = if item.open { "▾" } else { "▸" };
                Line::from(format!("{}{} {}", indent, arrow, item.name)).fg(ACCENT)
            } else {
                Line::from(format!("{}  {}", indent, item.name)).fg(TEXT)
            }
        }).collect();
        (" filer ".to_string(), lines)
    } else if searching {
        let mut lines: Vec<Line> = results.iter()
            .map(|(n, _)| Line::from(n.clone()).fg(TEXT))
            .collect();
        if results.is_empty() && !query.is_empty() {
            lines.push(Line::from(format!("Enter: lag nytt notat \"{}\"", query)).fg(ACCENT));
        }
        (format!(" søk: {}_ ", query), lines)
    } else {
        (" nylige notater ".to_string(), stats.top.iter().take((e * 10.0) as usize).cloned().collect())
    };

    // Highlight the selected line
    let lines: Vec<Line> = lines.into_iter().enumerate()
        .map(|(i, l)| if i == selected { l.fg(BG).bg(ACCENT) } else { l })
        .collect();

    // Scroll so the selected line is always visible
    let height = list_area.height.saturating_sub(2) as usize;
    let offset = selected.saturating_sub(height.saturating_sub(1));

    let list_box = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(title)
        .fg(MUTED)
        .bg(BG);
    frame.render_widget(Paragraph::new(lines).block(list_box).scroll((offset as u16, 0)), list_area);

    // Heatmap, GitHub style
    let mut legend = vec![Span::raw(" Less ").fg(MUTED)];
    for c in LEVELS {
        legend.push(Span::raw("■ ").fg(c));
    }
    legend.push(Span::raw("More ").fg(MUTED));

    let heat_box = Block::bordered()
        .border_type(BorderType::Rounded)
        .title(" aktivitet ")
        .title_bottom(Line::from(legend).right_aligned())
        .fg(MUTED)
        .bg(BG);

    let labels = ["Man ", "    ", "Ons ", "    ", "Fre ", "    ", "    "];

    // One row per weekday, one column per week, newest week on the right
    let heat: Vec<Line> = (0..7).map(|row| {
        let mut cells = vec![Span::raw(labels[row]).fg(MUTED)];
        for col in 0..WEEKS {
            let ago = (WEEKS - 1 - col) as i64 * 7 + today - row as i64;
            if ago < 0 {
                cells.push(Span::raw("   "));
                continue;
            }
            let count = stats.per_day[ago as usize];
            let level = match count {
                0 => 0,
                1 => 1,
                2 => 2,
                3..=4 => 3,
                _ => 4,
            };
            let level = if col as f32 > e * WEEKS as f32 { 0 } else { level };
            cells.push(Span::raw("▆▆ ").fg(LEVELS[level]));
        }
        Line::from(cells)
    }).collect();

    // Center the whole grid as one block, so the rows stay aligned
    let inner = heat_box.inner(heat_area);
    frame.render_widget(heat_box, heat_area);

    let grid_width = 4 + WEEKS as u16 * 3;
    let [grid_area] = Layout::horizontal([Constraint::Length(grid_width)])
        .flex(Flex::Center)
        .areas(inner);
    frame.render_widget(Paragraph::new(heat), grid_area);

    // Graph: the note under the cursor in the middle, its links around it, its tags inside,
    // and notes two steps away as faint dots. Hidden while the file tree is open.
    if tree.is_empty() {
        let focus_path: Option<&std::path::Path> = if searching {
            results.get(selected).map(|(_, path)| path.as_path())
        } else if prompt.is_none() {
            stats.paths.get(selected).map(|path| path.as_path())
        } else {
            None
        };
        let graph_box = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" graf ")
            .fg(MUTED)
            .bg(BG);

        if let Some(path) = focus_path {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let links: Vec<String> = stats.graph.get(&name).cloned().unwrap_or_default()
                .into_iter().take(10).collect();
            // Tags go along the bottom edge of the box, so they never cover the graph
            let tag_list: Vec<String> = stats.tags.get(&name).cloned().unwrap_or_default();
            let graph_box = if tag_list.is_empty() {
                graph_box
            } else {
                let text = tag_list.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join("  ");
                graph_box.title_bottom(Line::from(format!(" {} ", text)).fg(Color::Rgb(0x7C, 0x7D, 0x52)).right_aligned())
            };
            let tags: Vec<String> = Vec::new();

            // Notes two steps away, and which link leads to each of them
            let mut far: Vec<(String, usize)> = Vec::new();
            for (i, link) in links.iter().enumerate() {
                for next in stats.graph.get(link).cloned().unwrap_or_default() {
                    if next != name && !links.contains(&next) && !far.iter().any(|(n, _)| *n == next) {
                        far.push((next, i));
                    }
                }
            }
            far.clear();

            // Fly out again every time the note under the cursor changes
            let since = GRAPH_FOCUS.with(|focus| {
                let mut focus = focus.borrow_mut();
                if focus.0 != name {
                    *focus = (name.clone(), ms);
                }
                ms.saturating_sub(focus.1)
            });
            let fly = (since as f64 / 700.0).min(1.0);
            // Pops out a bit too far and settles back, like a mind map opening
            let back = 1.70158;
            let grow = e as f64 * (1.0 + (back + 1.0) * (fly - 1.0).powi(3) + back * (fly - 1.0).powi(2));

            let tau = std::f64::consts::TAU;
            let spin = 0.3; // fixed angles, no spinning
            let t = ms as f64 / 1000.0;
            let pulse = ((ms as f64 / 1000.0 * tau).sin() + 1.0) / 2.0;
            let centre = Color::Rgb(mix(0xA6, 0xE3, pulse as f32), mix(0xA8, 0xDC, pulse as f32), mix(0x67, 0x95, pulse as f32));
            let tag_color = Color::Rgb(0x7C, 0x7D, 0x52);
            let faint = Color::Rgb(0x3A, 0x40, 0x3C);

            // How wide one character is, in graph units, so names can stay inside the box
            let cell = 2.0 / graph_area.width.saturating_sub(2).max(1) as f64;

            // Places n things evenly on a circle
            let ring = move |count: usize, radius: f64, turn: f64| -> Vec<(f64, f64)> {
                (0..count).map(|i| {
                    let angle = turn + i as f64 / count.max(1) as f64 * tau;
                    // Every note hovers gently in its own rhythm
                    let bob = (t * 1.2 + i as f64 * 1.9).sin() * 0.03 * fly;
                    (angle.cos() * radius * grow, (angle.sin() * radius * 1.35).clamp(-0.95, 0.95) * grow + bob)
                }).collect()
            };

            let canvas = Canvas::default()
                .block(graph_box)
                .marker(Marker::Braille)
                .background_color(BG)
                .x_bounds([-1.0, 1.0])
                .y_bounds([-1.0, 1.0])
                .paint(move |ctx| {
                    let link_points = ring(links.len(), 0.62, spin);
                    let tag_points = ring(tags.len(), 0.32, -spin * 1.5 + 0.5);
                    let far_points = ring(far.len(), 0.95, spin * 0.6 + 0.2);

                    // Faint lines out to the notes two steps away, then the real links and tags
                    for (k, &(x, y)) in far_points.iter().enumerate() {
                        let (lx, ly) = link_points[far[k].1];
                        ctx.draw(&CanvasLine::new(lx, ly, x, y, faint));
                    }
                    for &(x, y) in &link_points {
                        ctx.draw(&CanvasLine::new(0.0, 0.0, x, y, MUTED));
                    }
                    for &(x, y) in &tag_points {
                        ctx.draw(&CanvasLine::new(0.0, 0.0, x, y, tag_color));
                    }
                    ctx.layer();
                    for &(x, y) in &far_points {
                        ctx.print(x, y, Span::raw("·").fg(MUTED));
                    }

                    // A name next to its dot, on the side that keeps it inside the box
                    let mut label = |x: f64, y: f64, dot: &str, text: String, color: Color| {
                        if x > 0.15 {
                            let width = (text.chars().count() + 1) as f64 * cell;
                            ctx.print(x - width, y, Span::raw(format!("{} {}", text, dot)).fg(color));
                        } else {
                            ctx.print(x, y, Span::raw(format!("{} {}", dot, text)).fg(color));
                        }
                    };

                    for (i, &(x, y)) in link_points.iter().enumerate() {
                        label(x, y, "●", short(&links[i]), ACCENT);
                    }
                    for (i, &(x, y)) in tag_points.iter().enumerate() {
                        // Keep tags off the line where the centre name is
                        let y = if y.abs() < 0.15 { y.signum() * 0.18 } else { y };
                        label(x, y, "#", short(&tags[i]), tag_color);
                    }
                    let centre_name = short(&name);
                    let half = (centre_name.chars().count() + 2) as f64 * cell / 2.0;
                    ctx.print(-half, 0.0, Span::raw(format!("◉ {}", centre_name)).fg(centre).bold());

                    if links.is_empty() && tags.is_empty() {
                        ctx.print(-0.35, -0.45, Span::raw("ingen lenker").fg(MUTED));
                    }
                });
            frame.render_widget(canvas, graph_area);
        } else {
            frame.render_widget(graph_box, graph_area);
        }
    }


    // Key hints for the mode we are in
    let keys: &[(&str, &str)] = if prompt.is_some() {
        &[("0-3", "velg mappe"), ("enter", "lag notat"), ("esc", "avbryt")]
    } else if !tree.is_empty() {
        &[("j/k", "flytt"), ("l", "åpne"), ("h", "lukk"), ("/", "søk"), ("esc", "tilbake"), ("q", "avslutt")]
    } else if searching {
        &[("↑/↓", "flytt"), ("enter", "åpne eller lag nytt"), ("esc", "tilbake")]
    } else {
        &[("j/k", "flytt"), ("enter", "åpne"), ("/", "søk"), ("e", "filer"), ("n", "nytt"), ("q", "avslutt")]
    };

    let mut spans = Vec::new();
    for (key, what) in keys {
        spans.push(Span::raw(format!(" {} ", key)).fg(BG).bg(ACCENT));
        spans.push(Span::raw(format!(" {}   ", what)).fg(MUTED));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)).bg(BG).centered(), help_area);
}

// Shortens long note names so the graph stays readable
fn short(name: &str) -> String {
    let cut: String = name.chars().take(16).collect();
    if name.chars().count() > 16 { format!("{}…", cut) } else { cut }
}

thread_local! {
    // The note the graph showed last time, and when it changed (for the fly-out animation)
    static GRAPH_FOCUS: std::cell::RefCell<(String, u32)> = std::cell::RefCell::new((String::new(), 0));
}
