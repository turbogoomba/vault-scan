use chrono::{Local, NaiveDate};

// Name and date of each exam
const EXAMS: [(&str, &str); 4] = [
    ("MEK2000 Matematikk", "2026-11-20"),
    ("ELI2300 Dynamiske systemer", "2026-11-23"),
    ("ELI2300 Prosjekt", "2026-11-26"),
    ("MEK3100 Python", "2026-12-15"),
];

// Exams that are not over yet, soonest first: (name, date, days left)
pub fn upcoming() -> Vec<(&'static str, String, i64)> {
    let today = Local::now().date_naive();

    let mut list: Vec<(&str, String, i64)> = EXAMS.iter().filter_map(|(name, date)| {
        let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
        let left = (day - today).num_days();
        if left < 0 {
            return None;
        }
        Some((*name, day.format("%d.%m").to_string(), left))
    }).collect();

    list.sort_by_key(|e| e.2);
    list
}
