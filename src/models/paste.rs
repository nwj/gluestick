use crate::db;
use rusqlite::named_params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Paste {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub body: String,
}

impl Paste {
    fn from_row(row: &rusqlite::Row) -> Result<Paste, tokio_rusqlite::Error> {
        Ok(Paste {
            id: row.get("id")?,
            title: row.get("title")?,
            description: row.get("description")?,
            body: row.get("body")?,
        })
    }

    pub async fn all(db: &db::Database) -> Result<Vec<Paste>, tokio_rusqlite::Error> {
        let pastes = db
            .conn
            .call(|conn| {
                let mut statement =
                    conn.prepare("SELECT id, title, description, body FROM pastes;")?;
                let mut rows = statement.query([])?;
                let mut pastes = Vec::new();
                while let Some(row) = rows.next()? {
                    pastes.push(Paste::from_row(row)?);
                }
                Ok(pastes)
            })
            .await?;

        Ok(pastes)
    }

    pub async fn insert(
        db: &db::Database,
        title: String,
        description: String,
        body: String,
    ) -> Result<usize, tokio_rusqlite::Error> {
        let result = db
            .conn
            .call(move |conn| {
                let id = Uuid::now_v7();
                let mut statement =
                    conn.prepare("INSERT INTO pastes VALUES (:id, :title, :description, :body);")?;
                let result = statement.execute(
                    named_params! {":id": id, ":title": title, ":description": description, ":body": body},
                )?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub async fn find(db: &db::Database, id: Uuid) -> Result<Option<Paste>, tokio_rusqlite::Error> {
        let maybe_paste = db
            .conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("SELECT id, title, description, body FROM pastes WHERE id = :id;")?;
                let mut rows = statement.query(named_params! {":id": id})?;
                if let Some(row) = rows.next()? {
                    Ok(Some(Paste::from_row(row)?))
                } else {
                    Ok(None)
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
