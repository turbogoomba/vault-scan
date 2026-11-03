use std::fs;
use std::path::Path;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::widgets::{Block, Paragraph};

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

fn main() {
    let vault = Path::new("/home/tallman/Documents/Main (No Nextcloud)");
    let total = scan(vault);

    let mut terminal = ratatui::init();
    loop {
        terminal.draw(|frame| {
            let block = Block::bordered().title(" vault-scan ");
            let text = Paragraph::new(format!("Ord: {}", total)).block(block);
            frame.render_widget(text, frame.area());
        }).unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }
    ratatui::restore();
}
