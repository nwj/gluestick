use crate::prelude::*;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use core::ops::RangeInclusive;
use rand::distributions::{Alphanumeric, DistString, Standard};
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::{thread_rng, Rng};
use rand_chacha::ChaCha20Rng;
use sha2::{Digest, Sha256};

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

pub fn random_api_key() -> String {
    let mut rng = ChaCha20Rng::from_entropy();
    format!("{:032x}", rng.gen::<u128>())
}

pub fn hash_password(password: String) -> Result<String> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))?
        .to_string())
}

pub fn hash_api_key(api_key: String) -> Vec<u8> {
    Sha256::digest(api_key.as_bytes()).to_vec()
}
