use crate::db;
use chrono::{
    serde::ts_seconds,
    {DateTime, Utc},
};
use rusqlite::named_params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Paste {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: String,
    pub body: String,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Paste {
    pub async fn all(db: &db::Database) -> Result<Vec<Paste>, tokio_rusqlite::Error> {
        let pastes = db
            .conn
            .call(|conn| {
                let mut statement = conn.prepare(
                    "SELECT id, user_id, title, description, body, created_at, updated_at FROM pastes;",
                )?;
                let results = serde_rusqlite::from_rows::<Paste>(statement.query([])?);
                let mut pastes = Vec::new();
                for result in results {
                    let paste = result.map_err(|e| tokio_rusqlite::Error::Other(Box::new(e)))?;
                    pastes.push(paste);
                }
                Ok(pastes)
            })
            .await?;

        Ok(pastes)
    }

    pub async fn insert(
        db: &db::Database,
        user_id: Uuid,
        title: String,
        description: String,
        body: String,
    ) -> Result<Uuid, tokio_rusqlite::Error> {
        let result = db
            .conn
            .call(move |conn| {
                let id = Uuid::now_v7();
                let mut statement =
                    conn.prepare("INSERT INTO pastes VALUES (:id, :user_id, :title, :description, :body, unixepoch(), unixepoch());")?;
                statement.execute(
                    named_params! {":id": id, ":user_id": user_id, ":title": title, ":description": description, ":body": body},
                )?;
                Ok(id)
            })
            .await?;

        Ok(result)
    }

    pub async fn find(db: &db::Database, id: Uuid) -> Result<Option<Paste>, tokio_rusqlite::Error> {
        let maybe_paste = db
            .conn
            .call(move |conn| {
                let mut statement = conn
                    .prepare("SELECT id, user_id, title, description, body, created_at, updated_at FROM pastes WHERE id = :id;")?;
                let mut rows = statement.query(named_params! {":id": id})?;
                match rows.next()? {
                    Some(row) => Ok(Some(
                        serde_rusqlite::from_row(row)
                            .map_err(|e| tokio_rusqlite::Error::Other(Box::new(e)))?,
                    )),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(maybe_paste)
    }

    pub async fn delete(db: &db::Database, id: Uuid) -> Result<usize, tokio_rusqlite::Error> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare("DELETE FROM pastes WHERE id = :id;")?;
                let result = statement.execute(named_params! {":id": id})?;
                Ok(result)
            })
            .await?;
        Ok(result)
    }
}
