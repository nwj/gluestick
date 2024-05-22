use crate::{db::Database, models};
use chrono::{
    serde::ts_seconds,
    {DateTime, Utc},
};
use rusqlite::{
    named_params,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Paste {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: String,
    pub body: String,
    pub visibility: Visibility,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Paste {
    pub fn new(
        user_id: Uuid,
        title: String,
        description: String,
        body: String,
        visibility: Visibility,
    ) -> Self {
        let id = Uuid::now_v7();
        let created_at = Utc::now();
        let updated_at = Utc::now();

        Self {
            id,
            user_id,
            title,
            description,
            body,
            visibility,
            created_at,
            updated_at,
        }
    }

    pub async fn all(db: &Database) -> models::Result<Vec<Paste>> {
        let pastes = db
            .conn
            .call(|conn| {
                let mut statement = conn.prepare(
                    "SELECT id, user_id, title, description, body, visibility, created_at, updated_at FROM pastes WHERE visibility = 'public';",
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

    pub async fn insert(self, db: &Database) -> models::Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    r"INSERT INTO pastes
                    VALUES (:id, :user_id, :title, :description, :body, :visibility, :created_at, :updated_at);"
                )?;
                let result = statement.execute(
                    named_params! {
                        ":id": self.id,
                        ":user_id": self.user_id,
                        ":title": self.title,
                        ":description": self.description,
                        ":body": self.body,
                        ":visibility": self.visibility,
                        ":created_at": self.created_at.timestamp(),
                        ":updated_at": self.updated_at.timestamp(),
                    }
                )?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub async fn find(db: &Database, id: Uuid) -> models::Result<Option<Paste>> {
        let optional_paste = db
            .conn
            .call(move |conn| {
                let mut statement = conn
                    .prepare("SELECT id, user_id, title, description, body, visibility, created_at, updated_at FROM pastes WHERE id = :id;")?;
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

        Ok(optional_paste)
    }

    pub async fn update(self, db: &Database) -> models::Result<usize> {
        let result = db.conn.call(move |conn| {
            let mut statement = conn.prepare(
                r"UPDATE pastes
                SET title = :title, description = :desc, body = :body, updated_at = unixepoch()
                WHERE id = :id;"
            )?;
            let result = statement.execute(named_params! {":title": self.title, ":desc": self.description, ":body": self.body, ":id": self.id})?;
            Ok(result)
        }).await?;
        Ok(result)
    }

    pub async fn delete(self, db: &Database) -> models::Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare("DELETE FROM pastes WHERE id = :id;")?;
                let result = statement.execute(named_params! {":id": self.id})?;
                Ok(result)
            })
            .await?;
        Ok(result)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Visibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "secret")]
    Secret,
}

impl ToSql for Visibility {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        match self {
            Visibility::Public => Ok("public".into()),
            Visibility::Secret => Ok("secret".into()),
        }
    }
}

impl FromSql for Visibility {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).and_then(|as_string| match as_string.as_str() {
            "public" => Ok(Visibility::Public),
            "secret" => Ok(Visibility::Secret),
            _ => Err(FromSqlError::Other(
                "Unrecognized value for visibility".into(),
            )),
        })
    }
}
