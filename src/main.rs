use std::fs;
use std::path::Path;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::{Block, BarChart, BorderType};
use ratatui::layout::Direction;
use ratatui::style::{Color, Style, Stylize};

const BG: Color = Color::Rgb(0x1B, 0x20, 0x21);
const MUTED: Color = Color::Rgb(0x51, 0x51, 0x3D);
const ACCENT: Color = Color::Rgb(0xA6, 0xA8, 0x67);
const TEXT: Color = Color::Rgb(0xE3, 0xDC, 0x95);

// To count all the words in the vault
fn scan(dir: &Path) -> u32 {
    let mut count = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            count += scan(&path);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let text = fs::read_to_string(&path).unwrap();
            count += text.split_whitespace().count() as u32;
        }
    }
    count
}

fn folders(vault: &Path) -> Vec<(String, u64)> {
    let mut result = Vec::new();
    for entry in fs::read_dir(vault).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() && !name.starts_with('.') {
            result.push((name, scan(&path) as u64));
        }
    }
    result
}

fn main() {
    let vault = Path::new("/home/tallman/Documents/Main (No Nextcloud)");
    let total = scan(vault);
    let data = folders(vault);
    let bars: Vec<(&str, u64)> = data.iter().map(|(n, c)| (n.as_str(), *c)).collect();

    let mut terminal = ratatui::init();
    loop {
        terminal.draw(|frame| {
            let block = Block::bordered()
                .border_type(BorderType::Rounded)
                .title(format!(" vault-scan   {} ord", total))
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
            frame.render_widget(chart, frame.area());

        }).unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }
    ratatui::restore();
}
