use std::path::Path;
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

    let mut terminal = ratatui::init();
    let start = Instant::now();
    loop {
        // Animation progress
        let p = (start.elapsed().as_secs_f32() / 2.0).min(1.0);
        let e = 1.0 - (1.0 - p).powi(3);
        let ms = start.elapsed().as_millis() as u32;

        terminal.draw(|frame| ui::draw(frame, &stats, p, e, ms)).unwrap();

        // Redraw often so the stars twinkle smoothly
        if event::poll(Duration::from_millis(if p < 1.0 { 16 } else { 33 })).unwrap() {
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
