use crate::models::{paste::Paste, session::Session, user::Username};
use askama_axum::Template;

#[derive(Template)]
#[template(path = "pastes/new.html")]
pub struct NewPastesTemplate {
    pub session: Option<Session>,
}

#[derive(Template)]
#[template(path = "pastes/index.html")]
pub struct IndexPastesTemplate {
    pub session: Option<Session>,
    pub paste_username_pairs: Vec<(Paste, Username)>,
}

#[derive(Template)]
#[template(path = "pastes/show.html")]
pub struct ShowPastesTemplate {
    pub session: Option<Session>,
    pub paste: Paste,
    pub username: Username,
}

#[derive(Template)]
#[template(path = "pastes/edit.html")]
pub struct EditPastesTemplate {
    pub session: Option<Session>,
    pub paste: Paste,
}

mod filters {
    use chrono::{DateTime, Duration, TimeZone, Utc};

    fn linewise_truncate_raw<T: std::fmt::Display>(s: T, n: usize, suffix: &str) -> String {
        let s = s.to_string();
        let mut lines = s.lines();
        let mut result = String::new();

        for _ in 0..n - 1 {
            if let Some(line) = lines.next() {
                result.push_str(line);
                result.push('\n');
            } else {
                return result;
            }
        }

        if let Some(last_line) = lines.next() {
            result.push_str(last_line.trim_end());
            result.push_str(suffix);
        }

        result
    }

    pub fn linewise_truncate<T: std::fmt::Display>(s: T, n: usize) -> askama::Result<String> {
        Ok(linewise_truncate_raw(s, n, "..."))
    }

    pub fn linewise_truncate_syntax_highlight<T: std::fmt::Display>(
        s: T,
    ) -> askama::Result<String> {
        Ok(linewise_truncate_raw(s, 11, "...</span></pre>"))
    }

    pub fn format_byte_size<T: std::fmt::Display>(s: T) -> askama::Result<String> {
        let s = s.to_string();
        let bytes = s.len();

        const KB: usize = 1024;
        const MB: usize = KB * 1024;
        const GB: usize = MB * 1024;
        const BYTE_LIMIT: usize = KB - 1;
        const KB_LIMIT: usize = MB - 1;
        const MB_LIMIT: usize = GB - 1;

        let size = match bytes {
            0..=BYTE_LIMIT => format!("{} bytes", bytes),
            KB..=KB_LIMIT => format!("{:.1} kb", bytes as f64 / KB as f64),
            MB..=MB_LIMIT => format!("{:.1} mb", bytes as f64 / MB as f64),
            _ => format!("{:.1} gb", bytes as f64 / GB as f64),
        };

        Ok(size)
    }

    pub fn format_relative_time<Tz: TimeZone>(datetime: &DateTime<Tz>) -> askama::Result<String> {
        let now = Utc::now();
        let diff = now.signed_duration_since(datetime);

        let (value, unit) = if diff < Duration::minutes(1) {
            (diff.num_seconds(), "second")
        } else if diff < Duration::hours(1) {
            (diff.num_minutes(), "minute")
        } else if diff < Duration::days(1) {
            (diff.num_hours(), "hour")
        } else if diff < Duration::days(30) {
            (diff.num_days(), "day")
        } else if diff < Duration::days(365) {
            (diff.num_days() / 30, "month")
        } else {
            (diff.num_days() / 365, "year")
        };

        Ok(format!(
            "{} {} ago",
            value,
            if value == 1 {
                unit.to_string()
            } else {
                unit.to_string() + "s"
            }
        ))
    }
}
