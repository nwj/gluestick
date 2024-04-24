use rusqlite_migration::{AsyncMigrations, M};
use tokio_rusqlite::Connection;

#[derive(Clone)]
pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub async fn init() -> Result<Self, Error> {
        let conn = Connection::open_in_memory().await?;
        Ok(Self { conn })
    }
}

pub fn migrations() -> AsyncMigrations {
    AsyncMigrations::new(vec![M::up(include_str!("migrations/01-init.sql"))])
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    Rusqlite(#[from] rusqlite::Error),
    TokioRusqlite(#[from] tokio_rusqlite::Error),
    RusqliteMigration(#[from] rusqlite_migration::Error),
}
