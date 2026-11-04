use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode};

mod vault;
mod scan;
mod theme;
mod tree;
mod ui;
mod exams;

use scan::load;

fn main() {
    let Some(vault_path) = vault::find() else {
        eprintln!("No Obsidian vault found. Run: vault-scan /path/to/vault");
        return;
    };
    let vault = vault_path.as_path();
    let mut stats = load(vault);
    let mut last_scan = Instant::now();

    let mut selected = 0;
    let mut searching = false;
    let mut query = String::new();
    let mut browsing = false;
    let mut expanded: HashSet<PathBuf> = HashSet::new();
    let mut items: Vec<tree::Item> = Vec::new();

    let mut terminal = ratatui::init();
    let start = Instant::now();
    let mut quit = false;

    while !quit {
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

        terminal.draw(|frame| {
            ui::draw(frame, &stats, p, e, ms, selected, searching, &query, &results, &items)
        }).unwrap();

        // Wait for the first key, then handle every key that has piled up
        let mut timeout = Duration::from_millis(if p < 1.0 { 16 } else { 33 });
        while event::poll(timeout).unwrap() {
            timeout = Duration::ZERO;

            let Event::Key(key) = event::read().unwrap() else { continue };

            // The note to open in Neovim, if any
            let mut open: Option<PathBuf> = None;
            // Whether the file tree needs to be read again
            let mut rebuild = false;

            if browsing {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        browsing = false;
                        selected = 0;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        selected = (selected + 1).min(items.len().saturating_sub(1));
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        selected = selected.saturating_sub(1);
                    }
                    KeyCode::Enter | KeyCode::Char('l') => {
                        if let Some(item) = items.get(selected) {
                            if item.is_dir {
                                // Open or close the folder
                                if !expanded.remove(&item.path) {
                                    expanded.insert(item.path.clone());
                                }
                                rebuild = true;
                            } else {
                                open = Some(item.path.clone());
                            }
                        }
                    }
                    KeyCode::Char('h') => {
                        if let Some(item) = items.get(selected) {
                            // Close this folder, or the folder this note is in
                            let target = if item.is_dir && item.open {
                                item.path.clone()
                            } else {
                                item.path.parent().unwrap().to_path_buf()
                            };
                            expanded.remove(&target);
                            if let Some(i) = items.iter().position(|it| it.path == target) {
                                selected = i;
                            }
                            rebuild = true;
                        }
                    }
                    _ => {}
                }
            } else if searching {
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
                    KeyCode::Esc => {
                        browsing = false;
                        selected = 0;
                    }
                    KeyCode::Char('q') => quit = true,
                    KeyCode::Char('/') => {
                        browsing = false;
                        searching = true;
                        selected = 0;
                    }

                    KeyCode::Char('e') => {
                        browsing = true;
                        selected = 0;
                        rebuild = true;
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
                if !browsing {
                    selected = 0;
                }
                rebuild = true;
            }

            // Read the folders again only when something changed
            if !browsing {
                items.clear();
            } else if rebuild {
                items.clear();
                tree::build(vault, &expanded, 0, &mut items);
            }

            if quit {
                break;
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
