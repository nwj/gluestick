use crate::{
    helpers::pagination::OffsetPaginationResponse,
    models::{paste::Paste, session::Session, user::Username},
};
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
    pub paste_username_html_triples: Vec<(Paste, Username, Option<String>)>,
    pub pagination: OffsetPaginationResponse,
}

#[derive(Template)]
#[template(path = "pastes/show.html")]
pub struct ShowPastesTemplate {
    pub session: Option<Session>,
    pub paste: Paste,
    pub username: Username,
    pub syntax_highlighted_html: Option<String>,
}

#[derive(Template)]
#[template(path = "pastes/edit.html")]
pub struct EditPastesTemplate {
    pub session: Option<Session>,
    pub paste: Paste,
}

mod filters {
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use std::fmt::Write;

    pub fn linewise_truncate<T: std::fmt::Display>(s: T, n: usize) -> askama::Result<String> {
        let s = s.to_string();
        let mut lines = s.lines();
        let mut truncated = lines.by_ref().take(n - 1).collect::<Vec<_>>().join("\n");

        if let Some(last_line) = lines.next() {
            if lines.next().is_some() {
                write!(truncated, "\n{}...", last_line.trim_end()).ok();
            } else {
                write!(truncated, "\n{}", last_line).ok();
            }
        }

        Ok(truncated)
    }

    // This has many limitations, including:
    // - assumes that the input is properly formatted html
    // - does not perfectly account for "void" elements (like <br>, <img>, etc.)
    // - likely has some unhandled issues with escaped characters
    //
    // Basically, this is good enough for our limited use case, but shouldn't be treated like it's
    // a robust html parser.
    pub fn linewise_truncate_html<T: std::fmt::Display>(s: T, n: usize) -> askama::Result<String> {
        let s = s.to_string();

        if s.lines().count() <= n + 1 {
            return Ok(s);
        }

        let mut lines = s.lines();
        let mut truncated = String::new();
        let mut open_tags = Vec::new();

        for _ in 0..n {
            if let Some(line) = lines.next() {
                writeln!(truncated, "{}", line).ok();

                let mut tag_start = None;
                for (i, c) in line.char_indices() {
                    match c {
                        '<' => tag_start = Some(i),
                        '>' => {
                            if let Some(start) = tag_start {
                                let tag = &line[start + 1..i];
                                if tag.starts_with('/') {
                                    open_tags.pop();
                                } else if !tag.ends_with("/>") {
                                    let tag_name = tag.split_whitespace().next().unwrap_or(tag);
                                    open_tags.push(tag_name.to_lowercase());
                                }
                            }
                            tag_start = None;
                        }
                        _ => {}
                    }
                }
            } else {
                break;
            }
        }

        if let Some(last_line) = lines.next() {
            write!(truncated, "{}...", last_line.trim_end()).ok();
            while let Some(tag) = open_tags.pop() {
                write!(truncated, "</{}>", tag).ok();
            }
        }

        Ok(truncated)
    }

    // This wrapper function is a workaround for the fact that our code formatter for jinja html
    // breaks askama when it formats things like `{{ foo|filter(10)|safe }}`
    pub fn linewise_truncate_html_10<T: std::fmt::Display>(s: T) -> askama::Result<String> {
        linewise_truncate_html(s, 10)
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
