use rusqlite_migration::{AsyncMigrations, M};
use tokio_rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub async fn new(config: &crate::config::Config) -> Result<Self, tokio_rusqlite::Error> {
        let conn = Connection::open(config.database_path()).await?;
        Ok(Self { conn })
    }
}

pub fn migrations() -> AsyncMigrations {
    AsyncMigrations::new(vec![M::up(include_str!("migrations/01-init.sql"))])
}
