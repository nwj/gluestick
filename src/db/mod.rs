use crate::controllers::prelude::Error as ControllerError;
use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use rusqlite_migration::{AsyncMigrations, M};
use tokio_rusqlite::Connection;

#[derive(Clone, Debug, FromRef)]
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

#[async_trait]
impl<S> FromRequestParts<S> for Database
where
    Self: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ControllerError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}
