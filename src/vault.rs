use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// Finds the Obsidian vault to use
pub fn find() -> Option<PathBuf> {
    // 1. A path given on the command line
    if let Some(arg) = env::args().nth(1) {
        return Some(PathBuf::from(arg));
    }

    // 2. The folder we are in, or one of the folders above it
    let cwd = env::current_dir().ok()?;
    for dir in cwd.ancestors() {
        if is_vault(dir) {
            return Some(dir.to_path_buf());
        }
    }

    // 3. Obsidian's own list of vaults
    if let Some(path) = from_obsidian() {
        return Some(path);
    }

    // 4. Search the home folder
    let home = PathBuf::from(env::var("HOME").ok()?);
    search(&home, 3)
}

// Every Obsidian vault has a hidden .obsidian folder
fn is_vault(dir: &Path) -> bool {
    dir.join(".obsidian").is_dir()
}

// Reads the vault list Obsidian keeps, and picks the one that was open last
fn from_obsidian() -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;
    let files = [
        format!("{}/.config/obsidian/obsidian.json", home),
        format!("{}/.var/app/md.obsidian.Obsidian/config/obsidian/obsidian.json", home),
    ];

    for file in files {
        let Ok(text) = fs::read_to_string(&file) else { continue };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else { continue };
        let Some(vaults) = json["vaults"].as_object() else { continue };

        let mut fallback = None;
        for v in vaults.values() {
            let Some(p) = v["path"].as_str() else { continue };
            let path = PathBuf::from(p);
            if !path.is_dir() {
                continue;
            }
            if v["open"].as_bool() == Some(true) {
                return Some(path);
            }
            fallback.get_or_insert(path);
        }
        if fallback.is_some() {
            return fallback;
        }
    }
    None
}

// Looks for a vault in this folder and the folders below it
fn search(dir: &Path, depth: u32) -> Option<PathBuf> {
    if is_vault(dir) {
        return Some(dir.to_path_buf());
    }
    if depth == 0 {
        return None;
    }

    let mut dirs: Vec<PathBuf> = fs::read_dir(dir).ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && !p.file_name().unwrap().to_string_lossy().starts_with('.'))
        .collect();
    dirs.sort();

    for d in dirs {
        if let Some(found) = search(&d, depth - 1) {
            return Some(found);
        }
    }
    None
}
