use std::fs;
use std::path::Path;

// To find all the files in vault
fn scan(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            scan(&path);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            println!("{}", path.display());
        }
    }
}
// Function for calling the other ones
fn main() {
    let vault = Path::new("/home/tallman/Documents/Main (No Nextcloud)");
    scan(vault);
}
