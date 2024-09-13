pub mod filters {
    use jiff::{tz::TimeZone, SpanRound, Timestamp, Unit, Zoned};
    use std::fmt::Write;

    pub fn linewise_truncate<T: std::fmt::Display>(s: T, n: usize) -> askama::Result<String> {
        let s = s.to_string();
        let mut lines = s.lines();
        let mut truncated = lines.by_ref().take(n - 1).collect::<Vec<_>>().join("\n");

        if let Some(last_line) = lines.next() {
            if lines.next().is_some() {
                write!(truncated, "\n{}...", last_line.trim_end()).ok();
            } else {
                write!(truncated, "\n{last_line}").ok();
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
                writeln!(truncated, "{line}").ok();

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
                write!(truncated, "</{tag}>").ok();
            }
        }

        Ok(truncated)
    }

    // This wrapper function is a workaround for the fact that our code formatter for jinja html
    // breaks askama since it formats things like `{{ foo|filter(10)|safe }}` to (incorrectly)
    // `{{ foo|filter(10) |safe}}`
    pub fn linewise_truncate_html_10<T: std::fmt::Display>(s: T) -> askama::Result<String> {
        linewise_truncate_html(s, 10)
    }

    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    pub fn format_byte_size<T: std::fmt::Display>(s: T) -> askama::Result<String> {
        const UNIT: f64 = 1000.0;
        const SUFFIX: [&str; 5] = ["bytes", "KB", "MB", "GB", "TB"];

        let s = s.to_string();
        let size = s.len();
        if size == 1 {
            return Ok("1 byte".into());
        }

        let size = size as f64;
        let base = size.log10() / UNIT.log10();
        let result = format!("{:.1}", UNIT.powf(base - base.floor()),)
            .trim_end_matches(".0")
            .to_owned();
        Ok([&result, SUFFIX[base.floor() as usize]].join(" "))
    }

    pub fn format_timestamp(ts: &Timestamp) -> askama::Result<String> {
        let datetime = Zoned::new(*ts, TimeZone::UTC);
        Ok(datetime.strftime("%b %d, %Y at %-I:%M%P %Z").to_string())
    }

    #[expect(clippy::cast_lossless)]
    pub fn format_timestamp_relative(ts: &Timestamp) -> askama::Result<String> {
        let now = Zoned::new(Timestamp::now(), TimeZone::UTC);
        let then = Zoned::new(*ts, TimeZone::UTC);
        let timespan = then
            .until((Unit::Year, &now))
            .map_err(|e| askama::Error::Custom(Box::new(e)))?;

        let (value, unit) = if timespan.get_years() > 0 {
            (
                timespan
                    .round(SpanRound::new().smallest(Unit::Year).relative(&now))
                    .map_err(|e| askama::Error::Custom(Box::new(e)))?
                    .get_years() as i64,
                "year",
            )
        } else if timespan.get_months() > 0 {
            (
                timespan
                    .round(SpanRound::new().smallest(Unit::Month).relative(&then))
                    .map_err(|e| askama::Error::Custom(Box::new(e)))?
                    .get_months() as i64,
                "month",
            )
        } else if timespan.get_days() > 0 {
            (
                timespan
                    .round(SpanRound::new().smallest(Unit::Day).relative(&now))
                    .map_err(|e| askama::Error::Custom(Box::new(e)))?
                    .get_days() as i64,
                "day",
            )
        } else if timespan.get_hours() > 0 {
            (
                timespan
                    .round(Unit::Hour)
                    .map_err(|e| askama::Error::Custom(Box::new(e)))?
                    .get_hours() as i64,
                "hour",
            )
        } else if timespan.get_minutes() > 0 {
            (
                timespan
                    .round(Unit::Minute)
                    .map_err(|e| askama::Error::Custom(Box::new(e)))?
                    .get_minutes(),
                "minute",
            )
        } else {
            (
                timespan
                    .round(Unit::Second)
                    .map_err(|e| askama::Error::Custom(Box::new(e)))?
                    .get_seconds(),
                "second",
            )
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

    #[cfg(test)]
    mod tests {
        use super::*;

        mod format_timestamp_relative {
            use super::*;
            use jiff::Span;

            #[test]
            fn exact_timespans() {
                let now = Zoned::new(Timestamp::now(), TimeZone::UTC);

                let ts = now.checked_sub(Span::new().years(2)).unwrap().timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "2 years ago");

                let ts = now.checked_sub(Span::new().months(5)).unwrap().timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "5 months ago");

                let ts = now.checked_sub(Span::new().days(17)).unwrap().timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "17 days ago");

                let ts = now.checked_sub(Span::new().hours(12)).unwrap().timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "12 hours ago");

                let ts = now
                    .checked_sub(Span::new().minutes(42))
                    .unwrap()
                    .timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "42 minutes ago");
            }

            #[test]
            fn singular_timespans() {
                let now = Zoned::new(Timestamp::now(), TimeZone::UTC);

                let ts = now.checked_sub(Span::new().years(1)).unwrap().timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "1 year ago");

                let ts = now.checked_sub(Span::new().months(1)).unwrap().timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "1 month ago");

                let ts = now.checked_sub(Span::new().days(1)).unwrap().timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "1 day ago");

                let ts = now.checked_sub(Span::new().hours(1)).unwrap().timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "1 hour ago");

                let ts = now.checked_sub(Span::new().minutes(1)).unwrap().timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "1 minute ago");
            }

            #[test]
            fn rounded_timespans() {
                let now = Zoned::new(Timestamp::now(), TimeZone::UTC);

                let ts = now
                    .checked_sub(Span::new().years(3).months(2).days(18))
                    .unwrap()
                    .timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "3 years ago");

                let ts = now
                    .checked_sub(Span::new().months(1).days(24).hours(7))
                    .unwrap()
                    .timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "2 months ago");

                let ts = now
                    .checked_sub(Span::new().days(5).hours(7).minutes(22).seconds(18))
                    .unwrap()
                    .timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "5 days ago");

                let ts = now
                    .checked_sub(Span::new().hours(7).minutes(33).seconds(18))
                    .unwrap()
                    .timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "8 hours ago");

                let ts = now
                    .checked_sub(Span::new().minutes(39).seconds(18))
                    .unwrap()
                    .timestamp();
                assert_eq!(format_timestamp_relative(&ts).unwrap(), "39 minutes ago");
            }
        }
    }
}
