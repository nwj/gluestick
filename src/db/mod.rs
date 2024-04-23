use include_dir::{include_dir, Dir};
use rusqlite_migration::AsyncMigrations;
use tokio_rusqlite::Connection;

const MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/db/migrations");

#[derive(Clone)]
pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub async fn init() -> Result<Self, DatabaseError> {
        let mut conn = Connection::open_in_memory().await?;

        AsyncMigrations::from_directory(&MIGRATIONS_DIR)?
            .to_latest(&mut conn)
            .await?;

        Ok(Self { conn })
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum DatabaseError {
    Rusqlite(#[from] rusqlite::Error),
    TokioRusqlite(#[from] tokio_rusqlite::Error),
    RusqliteMigration(#[from] rusqlite_migration::Error),
}
