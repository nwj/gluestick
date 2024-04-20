use rusqlite::Connection;
use rusqlite_migration::Migrations;
use include_dir::{Dir,include_dir};
use std::sync::{Arc, Mutex};
use anyhow::Result;

const MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/db/migrations");

#[derive(Clone)]
pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn init() -> Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        Migrations::from_directory(&MIGRATIONS_DIR)?.to_latest(&mut conn)?;

        let conn = Arc::new(Mutex::new(conn));
        Ok(Self { conn })
    }
}
