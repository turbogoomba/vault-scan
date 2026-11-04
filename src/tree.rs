use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

// One line in the file tree
pub struct Item {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub open: bool,
}

// Lists the vault as a tree, only going into folders that are open
pub fn build(dir: &Path, expanded: &HashSet<PathBuf>, depth: usize, out: &mut Vec<Item>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir).unwrap()
        .map(|e| e.unwrap().path())
        .collect();

    // Folders first, then alphabetical
    entries.sort_by_cached_key(|p| (!p.is_dir(), p.file_name().unwrap().to_string_lossy().to_lowercase()));

    for path in entries {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let is_dir = path.is_dir();

        // Skip hidden files and anything that is not a note
        if name.starts_with('.') || (!is_dir && !name.ends_with(".md")) {
            continue;
        }

        let open = expanded.contains(&path);
        out.push(Item {
            path: path.clone(),
            name: name.trim_end_matches(".md").to_string(),
            depth,
            is_dir,
            open,
        });

        if is_dir && open {
            build(&path, expanded, depth + 1, out);
        }
    }
}
