use crate::prelude::*;
use core::ops::RangeInclusive;
use rand::distributions::{Alphanumeric, DistString, Standard};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

const COMMON_TLDS: [&str; 32] = [
    ".au", ".biz", ".br", ".ca", ".cn", ".co", ".com", ".cz", ".de", ".edu", ".fr", ".gov", ".gr",
    ".in", ".info", ".io", ".it", ".jp", ".net", ".nl", ".nz", ".org", ".pl", ".ru", ".sg",
    ".site", ".tr", ".tw", ".ua", ".uk", ".vn", ".xyz",
];

const COMMON_FILE_EXTENSIONS: [&str; 32] = [
    ".c", ".conf", ".cpp", ".cs", ".css", ".csv", ".diff", ".go", ".h", ".html", ".java", ".js",
    ".json", ".lisp", ".log", ".lua", ".m", ".mat", ".md", ".php", ".py", ".r", ".rb", ".rs",
    ".sh", ".sql", ".toml", ".tsv", ".txt", ".vb", ".xml", ".yaml",
];

pub fn random_string(range: RangeInclusive<usize>) -> Result<String> {
    if range.is_empty() {
        return Err("range cannot be empty".into());
    }

    let mut rng = thread_rng();
    let len: usize = rng.gen_range(range);
    Ok(Standard.sample_string(&mut rng, len))
}

pub fn random_alphanumeric_string(range: RangeInclusive<usize>) -> Result<String> {
    if range.is_empty() {
        return Err("range cannot be empty".into());
    }

    let mut rng = thread_rng();
    let len: usize = rng.gen_range(range);
    Ok(Alphanumeric.sample_string(&mut rng, len))
}

pub fn random_email(range: RangeInclusive<usize>) -> Result<String> {
    if range.is_empty() {
        return Err("range cannot be empty".into());
    }

    if *range.start() < 6 {
        return Err("range lower bound must be 6 or greater to accommodate the space required for a (non-intranet) email address".into());
    }

    let mut rng = thread_rng();
    let len = rng.gen_range(range);
    let tld = COMMON_TLDS
        .iter()
        .filter(|tld| tld.len() <= len - 3)
        .collect::<Vec<_>>()
        .choose(&mut rng)
        .copied()
        .copied()
        .unwrap_or(".com");
    let remaining_len = len - 1 - tld.len();
    let username_len = remaining_len / 2 + remaining_len % 2;
    let domain_len = remaining_len / 2;
    let username = random_alphanumeric_string(1..=username_len)?;
    let domain = random_alphanumeric_string(1..=domain_len)?;
    Ok(format!("{username}@{domain}{tld}"))
}

pub fn random_filename(range: RangeInclusive<usize>) -> Result<String> {
    if range.is_empty() {
        return Err("range cannot be empty".into());
    }

    if *range.start() < 1 {
        return Err("range lower bound must be 1 or greater to accommodate the space required for a filename".into());
    }

    let mut rng = thread_rng();
    let len = rng.gen_range(range);
    let extension = COMMON_FILE_EXTENSIONS
        .iter()
        .filter(|ext| ext.len() <= len - 1)
        .collect::<Vec<_>>()
        .choose(&mut rng)
        .copied()
        .copied()
        .unwrap_or_default();
    let remaining_len = len - extension.len();
    let name = random_alphanumeric_string(1..=remaining_len)?;
    Ok(format!("{name}{extension}"))
}
