use std::fs;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

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

fn hash(a: u32, b: u32, c: u32) -> u32 {
    let mut h = a.wrapping_mul(374761393) ^ b.wrapping_mul(668265263) ^ c.wrapping_mul(2246822519);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
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

struct Stats {
    total: u32,
    notes: u32,
    dirs: Vec<(String, u64)>,
    top: Vec<Line<'static>>,
    per_day: [u32; 371],
}

fn load(vault: &Path) -> Stats {
    let (total, notes) = scan(vault);

    let mut dirs = folders(vault);
    dirs.sort();

    let mut all = Vec::new();
    collect(vault, &mut all);
    all.sort_by(|a, b| b.words.cmp(&a.words));

    let top = all.iter().take(10).map(|n| {
        Line::from(format!("{:>6}    {}", n.words, n.name)).fg(TEXT)
    }).collect();

    let mut per_day = [0u32; 371];
    for n in &all {
        if (n.days_ago as usize) < per_day.len() {
            per_day[n.days_ago as usize] += 1;
        }
    }

    Stats { total, notes, dirs, top, per_day }
}

fn main() {
    let vault = Path::new("/home/tallman/Documents/Main (No Nextcloud)");
    let mut stats = load(vault);
    let mut last_scan = Instant::now();

    let mut terminal = ratatui::init();
    let start = Instant::now();
    loop {
        let p = (start.elapsed().as_secs_f32() / 2.0).min(1.0);
        let e = 1.0 - (1.0 - p).powi(3);
        let ms = start.elapsed().as_millis() as u32;
        terminal.draw(|frame| {
            let [top, bottom] = Layout::vertical([
                Constraint::Length(10),
                Constraint::Min(0),
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

            let [middle, heat_area] = Layout::vertical([
                Constraint::Min(0),
                Constraint::Length(9),
            ]).areas(bottom);

            let [chart_area, list_area] = Layout::horizontal([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ]).areas(middle);

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

            // Biggest notes
            let list_box = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(" største notater ")
                .fg(MUTED)
                .bg(BG);
            frame.render_widget(Paragraph::new(stats.top.iter().take((e * 10.0) as usize).cloned().collect::<Vec<_>>()).block(list_box), list_area);

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
                    let count = stats.per_day[day];
                    let color = if count == 0 || col as f32 > e * 53.0 {
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

        // Wait up to half a second for a key press
        if event::poll(Duration::from_millis(if p < 1.0 { 16 } else { 500 })).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Rescan the vault every 2 seconds
        if last_scan.elapsed() >= Duration::from_secs(2) {
            stats = load(vault);
            last_scan = Instant::now();
        }
    }
    ratatui::restore();
}
