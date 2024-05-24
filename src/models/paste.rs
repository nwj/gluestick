use crate::{db::Database, models, models::user::Username};
use chrono::{
    serde::ts_seconds,
    {DateTime, Utc},
};
use rusqlite::{
    named_params,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
    Row,
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
        Self {
            id: Uuid::now_v7(),
            user_id,
            title,
            description,
            body,
            visibility,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn from_sql_row(row: &Row) -> models::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            user_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            body: row.get(4)?,
            visibility: row.get(5)?,
            created_at: DateTime::from_timestamp(row.get(6)?, 0)
                .ok_or(models::Error::ParseDateTime)?,
            updated_at: DateTime::from_timestamp(row.get(7)?, 0)
                .ok_or(models::Error::ParseDateTime)?,
        })
    }

    pub async fn all(db: &Database) -> models::Result<Vec<Paste>> {
        let paste_results: Vec<_> = db
            .conn
            .call(|conn| {
                let mut statement = conn.prepare(
                    r"SELECT id, user_id, title, description, body, visibility, created_at, updated_at FROM pastes
                    WHERE visibility = 'public'
                    ORDER BY updated_at DESC;",
                )?;
                let paste_iter = statement.query_map([], |row| {Ok(Paste::from_sql_row(row))})?;
                Ok(paste_iter.collect::<Result<Vec<_>, _>>()?)
        })
        .await?;
        paste_results.into_iter().collect::<Result<Vec<_>, _>>()
    }

    pub async fn all_with_usernames(db: &Database) -> models::Result<Vec<(Paste, Username)>> {
        let paste_results: Vec<_> = db
            .conn
            .call(|conn| {
                let mut statement = conn.prepare(
                    r"SELECT
                      pastes.id,
                      pastes.user_id,
                      pastes.title,
                      pastes.description,
                      pastes.body,
                      pastes.visibility,
                      pastes.created_at,
                      pastes.updated_at,
                      users.username
                    FROM pastes JOIN users ON pastes.user_id = users.id
                    WHERE pastes.visibility = 'public'
                    ORDER BY pastes.updated_at DESC;",
                )?;
                let paste_iter = statement.query_map([], |row| {
                    let paste_result = Paste::from_sql_row(row);
                    let username_result = Username::from_sql_row(row, 8);
                    Ok((paste_result, username_result))
                })?;
                Ok(paste_iter.collect::<Result<Vec<_>, _>>()?)
            })
            .await?;

        paste_results
            .into_iter()
            .map(|(username_result, paste_result)| {
                username_result.and_then(|username| paste_result.map(|paste| (username, paste)))
            })
            .collect()
    }

    pub async fn insert(self, db: &Database) -> models::Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "INSERT INTO pastes VALUES (:id, :user_id, :title, :description, :body, :visibility, :created_at, :updated_at);"
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
        let optional_result = db
            .conn
            .call(move |conn| {
                let mut statement = conn
                    .prepare("SELECT id, user_id, title, description, body, visibility, created_at, updated_at FROM pastes WHERE id = :id;")?;
                let mut rows = statement.query(named_params! {":id": id})?;
                match rows.next()? {
                    Some(row) => Ok(Some(
                        Paste::from_sql_row(row)
                    )),
                    None => Ok(None),
                }
            })
            .await?;

        let optional_paste = optional_result.transpose()?;
        Ok(optional_paste)
    }

    pub async fn find_with_username(
        db: &Database,
        id: Uuid,
    ) -> models::Result<Option<(Paste, Username)>> {
        let optional_result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    r"SELECT
                          pastes.id,
                          pastes.user_id,
                          pastes.title,
                          pastes.description,
                          pastes.body,
                          pastes.visibility,
                          pastes.created_at,
                          pastes.updated_at,
                          users.username
                        FROM pastes JOIN users ON pastes.user_id = users.id
                        WHERE pastes.id = :id;",
                )?;
                let mut rows = statement.query(named_params! {":id": id})?;
                match rows.next()? {
                    Some(row) => Ok(Some((
                        Paste::from_sql_row(row),
                        Username::from_sql_row(row, 8),
                    ))),
                    None => Ok(None),
                }
            })
            .await?;

        optional_result
            .map(|(res_a, res_b)| res_a.and_then(|a| res_b.map(|b| (a, b))))
            .transpose()
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
