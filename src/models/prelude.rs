#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error(transparent)]
    TokioRusqlite(#[from] tokio_rusqlite::Error),

    #[error(transparent)]
    Garde(#[from] garde::error::Report),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
