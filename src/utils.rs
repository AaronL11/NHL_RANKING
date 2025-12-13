use chrono::Utc;

pub fn today_iso() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}
