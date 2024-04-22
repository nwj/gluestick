use crate::db::Database;
use anyhow::Result;
use rusqlite::named_params;

#[derive(Debug)]
pub struct Paste {
    pub id: i64,
    pub text: String,
}

impl Paste {
    fn from_row(row: &rusqlite::Row) -> Result<Paste, tokio_rusqlite::Error> {
        Ok(Paste {
            id: row.get("id")?,
            text: row.get("text")?,
        })
    }

    pub async fn all(db: &Database) -> Result<Vec<Paste>> {
        let pastes = db
            .conn
            .call(|conn| {
                let mut statement = conn.prepare("SELECT id, text FROM pastes;")?;
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

    pub async fn insert(db: &Database, text: String) -> Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare("INSERT INTO pastes (text) VALUES (:text);")?;
                let result = statement.execute(named_params! {":text": text})?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub async fn find(db: &Database, id: i64) -> Result<Option<Paste>> {
        let maybe_paste = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare("SELECT id, text FROM pastes WHERE id = :id;")?;
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

    pub async fn delete(db: &Database, id: i64) -> Result<usize> {
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
