use anyhow::Result;
use include_dir::{include_dir, Dir};
use rusqlite_migration::AsyncMigrations;
use tokio_rusqlite::Connection;

const MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/db/migrations");

#[derive(Clone)]
pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub async fn init() -> Result<Self> {
        let mut conn = Connection::open_in_memory().await?;
        AsyncMigrations::from_directory(&MIGRATIONS_DIR)?
            .to_latest(&mut conn)
            .await?;
        Ok(Self { conn })
    }
}
