use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use ratatui::style::Stylize;
use ratatui::text::Line;

use crate::theme::TEXT;

const INCLUDE: [&str; 3] = ["01", "02", "03"]; // What notes to include in stats

// Defines what a note is
struct Note {
    path: PathBuf,
    name: String,
    words: u32,
    days_ago: u64,
    links: Vec<String>,
    tags: Vec<String>,
    age: u64,
}

// Defines what the stats are
pub struct Stats {
    pub paths: Vec<PathBuf>,
    pub total: u32,
    pub notes: u32,
    pub dirs: Vec<(String, u64)>,
    pub top: Vec<Line<'static>>,
    pub per_day: [u32; 371],
    pub all: Vec<(String, PathBuf)>,
    pub graph: HashMap<String, Vec<String>>,
    pub tags: HashMap<String, Vec<String>>,
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
            let age = SystemTime::now()
                .duration_since(modified)
                .unwrap_or_default()
                .as_secs();
            let days_ago = age / 86400;

            out.push(Note {
                path: path.clone(),
                name: name.trim_end_matches(".md").to_string(),
                words: text.split_whitespace().count() as u32,
                days_ago,
                links: links_in(&text),
                tags: tags_in(&text),
                age,
            });
        }
    }
}

// Scans the vault and gathers everything the dashboard shows
// Every [[link]] in a note, without |alias, #heading or folders
fn links_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("[[") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("]]") else { break };
        let target = after[..end].split(['|', '#']).next().unwrap_or("");
        let target = target.rsplit('/').next().unwrap_or("").trim();
        if !target.is_empty() {
            out.push(target.to_string());
        }
        rest = &after[end + 2..];
    }
    out
}

// Every tag in a note, from the frontmatter (tags: [a, b] or a list) and #tags in the text
fn tags_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut lines = text.lines().peekable();

    if lines.peek().map(|l| l.trim()) == Some("---") {
        lines.next();
        let mut in_list = false;
        for line in lines.by_ref() {
            let l = line.trim();
            if l == "---" {
                break;
            }
            if let Some(rest) = l.strip_prefix("tags:") {
                let rest = rest.trim().trim_start_matches('[').trim_end_matches(']');
                for tag in rest.split(',') {
                    let tag = tag.trim().trim_start_matches('#');
                    if !tag.is_empty() {
                        out.push(tag.to_string());
                    }
                }
                in_list = rest.is_empty();
            } else if in_list && l.starts_with("- ") {
                out.push(l[2..].trim().trim_start_matches('#').to_string());
            } else {
                in_list = false;
            }
        }
    }

    // #tag in the text, but not headings (# Title)
    for line in lines {
        for word in line.split_whitespace() {
            if let Some(tag) = word.strip_prefix('#') {
                let tag: String = tag.chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '/')
                    .collect();
                if !tag.is_empty() && !tag.chars().all(|c| c.is_ascii_digit()) {
                    out.push(tag);
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

pub fn load(vault: &Path) -> Stats {
    let (total, notes) = scan(vault);

    let mut dirs = folders(vault);
    dirs.sort();

    let mut all = Vec::new();
    collect(vault, &mut all);

    // Every note, sorted by name, for searching
    let mut all_notes: Vec<(String, PathBuf)> = all.iter()
        .map(|n| (n.name.clone(), n.path.clone()))
        .collect();
    all_notes.sort();

    // Most recently changed notes first
    all.sort_by_key(|n| n.age);

    let top = all.iter().take(10).map(|n| {
        let when = match n.age {
            a if a < 3600 => format!("{} min", a / 60),
            a if a < 86400 => format!("{} t", a / 3600),
            a if a < 2 * 86400 => "i går".to_string(),
            a => format!("{} d", a / 86400),
        };
        Line::from(format!("{:>6}    {}", when, n.name)).fg(TEXT)
    }).collect();

    let paths = all.iter().take(10).map(|n| n.path.clone()).collect();

    let mut per_day = [0u32; 371];
    for n in &all {
        if (n.days_ago as usize) < per_day.len() {
            per_day[n.days_ago as usize] += 1;
        }
    }

    // Who links to whom, both ways, so every note knows its neighbours
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for n in &all {
        for target in &n.links {
            if *target == n.name {
                continue;
            }
            graph.entry(n.name.clone()).or_default().push(target.clone());
            graph.entry(target.clone()).or_default().push(n.name.clone());
        }
    }
    for list in graph.values_mut() {
        list.sort();
        list.dedup();
    }

    // Tags for every note
    let tags: HashMap<String, Vec<String>> = all.iter()
        .map(|n| (n.name.clone(), n.tags.clone()))
        .collect();

    Stats { total, notes, dirs, top, paths, per_day, all: all_notes, graph, tags }
}
