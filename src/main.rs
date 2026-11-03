use std::fs;
use std::path::Path;
use std::time::SystemTime;

use ratatui::text::Span;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::{Block, BarChart, BorderType, Paragraph};
use ratatui::layout::{Constraint, Direction, Flex, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;

const LOGO: &str = r#"

   :::     :::     :::     :::    ::: :::    :::::::::::          ::::::::   ::::::::      :::     ::::    :::
  :+:     :+:   :+: :+:   :+:    :+: :+:        :+:             :+:    :+: :+:    :+:   :+: :+:   :+:+:   :+:
 +:+     +:+  +:+   +:+  +:+    +:+ +:+        +:+             +:+        +:+         +:+   +:+  :+:+:+  +:+
+#+     +:+ +#++:++#++: +#+    +:+ +#+        +#+             +#++:++#++ +#+        +#++:++#++: +#+ +:+ +#+
+#+   +#+  +#+     +#+ +#+    +#+ +#+        +#+                    +#+ +#+        +#+     +#+ +#+  +#+#+#
#+#+#+#   #+#     #+# #+#    #+# #+#        #+#             #+#    #+# #+#    #+# #+#     #+# #+#   #+#+#
 ###     ###     ###  ########  ########## ###              ########   ########  ###     ### ###    ####

"#;

const INCLUDE: [&str; 3] = ["01", "02", "03"];

const BG: Color = Color::Rgb(0x1B, 0x20, 0x21);
const MUTED: Color = Color::Rgb(0x51, 0x51, 0x3D);
const ACCENT: Color = Color::Rgb(0xA6, 0xA8, 0x67);
const TEXT: Color = Color::Rgb(0xE3, 0xDC, 0x95);

struct Note {
    name: String,
    words: u32,
    days_ago: u64,
}

// To count all the words in the vault
fn scan(dir: &Path) -> (u32, u32) {
    let mut words = 0;
    let mut notes = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            let (w, n) = scan(&path);
            words += w;
            notes += n;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let text = fs::read_to_string(&path).unwrap();
            words += text.split_whitespace().count() as u32;
            notes += 1;
        }
    }
    (words, notes)
}

fn folders(vault: &Path) -> Vec<(String, u64)> {
    let mut result = Vec::new();
    for entry in fs::read_dir(vault).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() && !name.starts_with(".") && INCLUDE.iter().any(|b| name.starts_with(b)) {
            result.push((name, scan(&path).0 as u64));
        }
    }
    result
}

// Blend between two numbers, t goes from 0.0 to 1.0
fn mix(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t) as u8
}

fn collect(dir: &Path, out: &mut Vec<Note>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let text = fs::read_to_string(&path).unwrap();
            let modified= fs::metadata(&path).unwrap().modified().unwrap();
            let days_ago = SystemTime::now()
                .duration_since(modified)
                .unwrap_or_default()
                .as_secs() / 86400;

            out.push(Note {
                name: name.trim_end_matches(".md").to_string(),
                words: text.split_whitespace().count() as u32,
                days_ago,
            });
        }
    }
}

fn main() {
    let vault = Path::new("/home/tallman/Documents/Main (No Nextcloud)");
    let (total, notes) = scan(vault);
    let mut data = folders(vault);
    data.sort();
    let bars: Vec<(&str, u64)> = data.iter().map(|(n, c)| (n.as_str(), *c)).collect();

    let mut all = Vec::new();
    collect(vault, &mut all);

    all.sort_by(|a, b| b.words.cmp(&a.words));
    let top_notes: Vec<Line> = all.iter().take(10).map(|n| {
        Line::from(format!("{:>6}   {}", n.words, n.name)).fg(TEXT)
    }).collect();

    let mut per_day = [0u32; 371];
    for n in &all {
        if (n.days_ago as usize) < per_day.len() {
            per_day[n.days_ago as usize] += 1;
        }
    }

    let mut terminal = ratatui::init();
    loop {
        terminal.draw(|frame| {
            let [top, bottom] = Layout::vertical([
                Constraint::Length(10),
                Constraint::Min(0),
            ]).areas(frame.area());

            let [logo_area] = Layout::horizontal([Constraint::Length(110)])
                .flex(Flex::Center)
                .areas(top);

            // Gradient from MUTED at the top to TEXT at the bottom
            let rows: Vec<Line> = LOGO.lines().enumerate().map(|(i, line)| {
                let t = i as f32 / 9.0;
                let color = Color::Rgb(
                    mix(0x51, 0xE3, t),
                    mix(0x51, 0xDC, t),
                    mix(0x3D, 0x95, t),
                );
                Line::from(line).fg(color)
            }).collect();

            frame.render_widget(Block::new().bg(BG), top);
            frame.render_widget(Paragraph::new(rows).bg(BG), logo_area);

            let [middle, heat_area] = Layout::vertical([
                Constraint::Min(0),
                Constraint::Length(9),
            ]).areas(bottom);

            let [chart_area, list_area] = Layout::horizontal([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ]).areas(middle);

            // Bar chart
            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(format!(" vault-scan   {} words   {} notater ", total, notes))
                .fg(MUTED)
                .bg(BG);
            let chart = BarChart::default()
                .block(block)
                .data(&bars)
                .direction(Direction::Horizontal)
                .bar_width(1)
                .bar_gap(1)
                .bar_style(Style::new().fg(ACCENT))
                .value_style(Style::new().fg(BG).bg(ACCENT))
                .label_style(Style::new().fg(TEXT));
            frame.render_widget(chart, chart_area);

            // Biggest notes
            let list_box = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" største notater ")
                .fg(MUTED)
                .bg(BG);
            frame.render_widget(Paragraph::new(top_notes.clone()).block(list_box), list_area);

            // Heatmap
            let heat_box = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" aktivitet ")
                .fg(MUTED)
                .bg(BG);

            // One square per day, newest in the bottom right corner
            let heat: Vec<Line> = (0..7).map(|row| {
                let cells: Vec<Span> = (0..53).map(|col| {
                    let day = (52 - col) * 7 + (6 - row);
                    let count = per_day[day];
                    let color = if count == 0 {
                        Color::Rgb(0x2A, 0x30, 0x30)
                    } else {
                        let t = count.min(5) as f32 / 5.0;
                        Color::Rgb(mix(0x51, 0xE3, t), mix(0x51, 0xDC, t), mix(0x3D, 0x95, t))
                    };
                    Span::raw("██").fg(color)
                }).collect();
                Line::from(cells)
            }).collect();

            frame.render_widget(Paragraph::new(heat).block(heat_box).centered(), heat_area);
        }).unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }
    ratatui::restore();
}
