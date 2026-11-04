use std::process::Command;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode};

mod scan;
mod theme;
mod ui;

use scan::load;

fn main() {
    let vault = Path::new("/home/tallman/Documents/Main (No Nextcloud)");
    let mut stats = load(vault);
    let mut last_scan = Instant::now();
    let mut selected = 0;
    let mut searching = false;
    let mut query = String::new();

    let mut terminal = ratatui::init();
    let start = Instant::now();
    loop {
        // Animation progress
        let p = (start.elapsed().as_secs_f32() / 2.0).min(1.0);
        let e = 1.0 - (1.0 - p).powi(3);
        let ms = start.elapsed().as_millis() as u32;

        // Notes whose name contains the search text
        let results: Vec<(String, PathBuf)> = if searching {
            let q = query.to_lowercase();
            stats.all.iter()
                .filter(|(n, _)| n.to_lowercase().contains(&q))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };

        terminal.draw(|frame| ui::draw(frame, &stats, p, e, ms, selected, searching, &query, &results)).unwrap();

        // Redraw often so the stars twinkle smoothly
        if event::poll(Duration::from_millis(if p < 1.0 { 16 } else { 33 })).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                // The note to open in Neovim, if any
                let mut open: Option<PathBuf> = None;

                if searching {
                    match key.code {
                        KeyCode::Esc => {
                            searching = false;
                            query.clear();
                            selected = 0;
                        }
                        KeyCode::Char(c) => {
                            query.push(c);
                            selected = 0;
                        }
                        KeyCode::Backspace => {
                            query.pop();
                            selected = 0;
                        }
                        KeyCode::Down => selected = (selected + 1).min(results.len().saturating_sub(1)),
                        KeyCode::Up => selected = selected.saturating_sub(1),
                        KeyCode::Enter => {
                            if let Some((_, path)) = results.get(selected) {
                                open = Some(path.clone());
                            } else if !query.is_empty() {
                                // No match, so make a new note in the vault root
                                open = Some(vault.join(format!("{}.md", query)));
                            }
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('/') => {
                            searching = true;
                            selected = 0;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            selected = (selected + 1).min(stats.paths.len().saturating_sub(1));
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            selected = selected.saturating_sub(1);
                        }
                        KeyCode::Enter => open = stats.paths.get(selected).cloned(),
                        _ => {}
                    }
                }

                // Hand the terminal to Neovim, then take it back
                if let Some(path) = open {
                    ratatui::restore();
                    Command::new("nvim").arg(&path).status().unwrap();
                    terminal = ratatui::init();
                    stats = load(vault);
                    searching = false;
                    query.clear();
                    selected = 0;
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
