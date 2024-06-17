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
        return Err("range lower bound must be 6 or greater to accomodate the space required for a (non-intranet) email address".into());
    }

    let mut rng = thread_rng();
    let len = rng.gen_range(range);
    let tld = match len {
        6 => COMMON_TLDS
            .iter()
            .copied()
            .filter(|tld| tld.len() <= 3)
            .collect::<Vec<_>>()
            .choose(&mut rng)
            .copied()
            .unwrap_or(".co"),
        7 => COMMON_TLDS
            .iter()
            .copied()
            .filter(|tld| tld.len() <= 4)
            .collect::<Vec<_>>()
            .choose(&mut rng)
            .copied()
            .unwrap_or(".com"),
        _ => COMMON_TLDS.choose(&mut rng).unwrap(),
    };
    let remaining_len = len - 1 - tld.len();
    let username_len = remaining_len / 2 + remaining_len % 2;
    let domain_len = remaining_len / 2;
    let username = random_alphanumeric_string(1..=username_len)?;
    let domain = random_alphanumeric_string(1..=domain_len)?;
    Ok(format!("{username}@{domain}{tld}"))
}
