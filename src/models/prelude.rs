#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),

    #[error(transparent)]
    TokioRusqlite(#[from] tokio_rusqlite::Error),

    #[error(transparent)]
    Jiff(#[from] jiff::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
