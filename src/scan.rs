use std::fs;
use std::path::Path;
use std::time::SystemTime;

use ratatui::style::Stylize;
use ratatui::text::Line;

use crate::theme::TEXT;

const INCLUDE: [&str; 3] = ["01", "02", "03"]; // What notes to include in stats

// Defines what a note is
struct Note {
    name: String,
    words: u32,
    days_ago: u64,
}

// Defines what the stats are
pub struct Stats {
    pub total: u32,
    pub notes: u32,
    pub dirs: Vec<(String, u64)>,
    pub top: Vec<Line<'static>>,
    pub per_day: [u32; 371],
}

// The scan function
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

// Function to find folders in vault
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

// This is dont know what does but it kinda collects the files
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

// This finds the vault
pub fn load(vault: &Path) -> Stats {
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
