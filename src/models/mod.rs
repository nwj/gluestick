pub mod api_session;
pub mod paste;
pub mod session;
pub mod user;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    #[error(transparent)]
    TokioRusqlite(#[from] tokio_rusqlite::Error),

    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),

    #[error("failed to parse i64 to DateTime")]
    ParseDateTime,

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}
