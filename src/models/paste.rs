use crate::{
    db::Database,
    helpers::{
        pagination::{Direction, HasOrderedId},
        syntax_highlight,
    },
    models,
    models::user::Username,
};
use chrono::{
    serde::ts_seconds,
    {DateTime, Utc},
};
use derive_more::{AsRef, Display, Into};
use rusqlite::{
    named_params,
    types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Type, ValueRef},
    Row, Transaction, TransactionBehavior,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paste {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: Filename,
    pub description: Description,
    pub body: Body,
    pub visibility: Visibility,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Paste {
    pub fn new(
        user_id: Uuid,
        filename: String,
        description: String,
        body: String,
        visibility: Visibility,
    ) -> models::Result<Self> {
        Ok(Self {
            id: Uuid::now_v7(),
            user_id,
            filename: Filename::try_from(filename)?,
            description: Description::try_from(description)?,
            body: Body::try_from(body)?,
            visibility,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn from_sql_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            user_id: row.get(1)?,
            filename: row.get(2)?,
            description: row.get(3)?,
            body: row.get(4)?,
            visibility: row.get(5)?,
            created_at: DateTime::from_timestamp(row.get(6)?, 0).ok_or(
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    Type::Integer,
                    Box::new(models::Error::ParseDateTime),
                ),
            )?,
            updated_at: DateTime::from_timestamp(row.get(7)?, 0).ok_or(
                rusqlite::Error::FromSqlConversionFailure(
                    7,
                    Type::Integer,
                    Box::new(models::Error::ParseDateTime),
                ),
            )?,
        })
    }

    pub async fn syntax_highlight(&self, db: &Database) -> models::Result<Option<String>> {
        Ok(syntax_highlight::generate_with_cache_attempt(
            db,
            &self.id,
            self.body.as_ref(),
            self.filename.extension(),
        )
        .await?)
    }

    pub async fn cursor_paginated(
        db: &Database,
        limit: usize,
        direction: Direction,
        cursor: Option<Uuid>,
    ) -> models::Result<Vec<Paste>> {
        let pastes: Vec<_> = db
            .conn
            .call(move |conn| {
                let direction_sql = direction.to_raw_sql();
                let cursor_sql = match (cursor, &direction) {
                    (None, _) => "",
                    (Some(_), Direction::Ascending) => "AND pastes.id > :cursor",
                    (Some(_), Direction::Descending) => "AND pastes.id < :cursor",
                };
                let raw_sql = format!(
                    r"SELECT id, user_id, filename, description, body, visibility, created_at, updated_at
                    FROM pastes WHERE visibility = 'public' {cursor_sql} ORDER BY pastes.id {direction_sql} LIMIT :limit;"
                );
                let mut stmt = conn.prepare(&raw_sql)?;
                match cursor {
                    None => {
                        let paste_iter = stmt.query_map(named_params! {":limit": limit}, Paste::from_sql_row)?;
                        Ok(paste_iter.collect::<Result<Vec<_>, _>>()?)
                    }
                    Some(cursor) => {
                        let paste_iter = stmt.query_map(named_params! {":limit": limit, ":cursor": cursor}, Paste::from_sql_row)?;
                        Ok(paste_iter.collect::<Result<Vec<_>, _>>()?)
                    }
                }
            })
            .await?;
        Ok(pastes)
    }

    pub async fn cursor_paginated_with_username(
        db: &Database,
        limit: usize,
        direction: Direction,
        cursor: Option<Uuid>,
    ) -> models::Result<Vec<(Paste, Username)>> {
        let pairs: Vec<_> = db
            .conn
            .call(move |conn| {
                let direction_sql = direction.to_raw_sql();
                let cursor_sql = match (cursor, &direction) {
                    (None, _) => "",
                    (Some(_), Direction::Ascending) => "AND pastes.id > :cursor",
                    (Some(_), Direction::Descending) => "AND pastes.id < :cursor",
                };
                let raw_sql = format!(
                    r"SELECT
                      pastes.id,
                      pastes.user_id,
                      pastes.filename,
                      pastes.description,
                      pastes.body,
                      pastes.visibility,
                      pastes.created_at,
                      pastes.updated_at,
                      users.username
                    FROM pastes JOIN users ON pastes.user_id = users.id
                    WHERE pastes.visibility = 'public' {cursor_sql}
                    ORDER BY pastes.id {direction_sql}
                    LIMIT :limit;"
                );
                let mut stmt = conn.prepare(&raw_sql)?;
                match cursor {
                    None => {
                        let paste_iter =
                            stmt.query_map(named_params! {":limit": limit}, |row| {
                                let paste_result = Paste::from_sql_row(row)?;
                                let username: Username = row.get(8)?;
                                Ok((paste_result, username))
                            })?;
                        Ok(paste_iter.collect::<Result<Vec<_>, _>>()?)
                    }
                    Some(cursor) => {
                        let paste_iter = stmt.query_map(
                            named_params! {":cursor": cursor, ":limit": limit},
                            |row| {
                                let paste_result = Paste::from_sql_row(row)?;
                                let username: Username = row.get(8)?;
                                Ok((paste_result, username))
                            },
                        )?;
                        Ok(paste_iter.collect::<Result<Vec<_>, _>>()?)
                    }
                }
            })
            .await?;

        Ok(pairs)
    }

    pub async fn insert(self, db: &Database) -> models::Result<()> {
        let optional_html =
            syntax_highlight::generate(self.body.as_ref(), self.filename.extension());

        db.conn
            .call(move |conn| {
                let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
                {
                    let mut stmt = tx.prepare(
                        "INSERT INTO pastes VALUES (:id, :user_id, :filename, :description, :body, :visibility, :created_at, :updated_at);"
                    )?;
                    stmt.execute(
                        named_params! {
                            ":id": self.id,
                            ":user_id": self.user_id,
                            ":filename": self.filename,
                            ":description": self.description,
                            ":body": self.body,
                            ":visibility": self.visibility,
                            ":created_at": self.created_at.timestamp(),
                            ":updated_at": self.updated_at.timestamp(),
                        }
                    )?;

                    if let Some(html) = optional_html {
                        syntax_highlight::tx_cache_set(&tx, &self.id, &html)?;
                    }
                }
                tx.commit()?;

                Ok(())
            })
            .await?;

        Ok(())
    }

    pub async fn find(db: &Database, id: Uuid) -> models::Result<Option<Paste>> {
        let optional_paste = db
            .conn
            .call(move |conn| {
                let mut stmt = conn
                    .prepare("SELECT id, user_id, filename, description, body, visibility, created_at, updated_at FROM pastes WHERE id = :id;")?;
                let mut rows = stmt.query(named_params! {":id": id})?;
                match rows.next()? {
                    Some(row) => Ok(Some(
                        Paste::from_sql_row(row)?
                    )),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_paste)
    }

    pub fn tx_find(tx: &Transaction, id: &Uuid) -> tokio_rusqlite::Result<Option<Paste>> {
        let mut stmt = tx.prepare("SELECT id, user_id, filename, description, body, visibility, created_at, updated_at FROM pastes WHERE id = :id;")?;
        let mut rows = stmt.query(named_params! {":id": id})?;
        match rows.next()? {
            Some(row) => Ok(Some(Paste::from_sql_row(row)?)),
            None => Ok(None),
        }
    }

    pub async fn find_with_username(
        db: &Database,
        id: Uuid,
    ) -> models::Result<Option<(Paste, Username)>> {
        let optional_pair = db
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    r"SELECT
                          pastes.id,
                          pastes.user_id,
                          pastes.filename,
                          pastes.description,
                          pastes.body,
                          pastes.visibility,
                          pastes.created_at,
                          pastes.updated_at,
                          users.username
                        FROM pastes JOIN users ON pastes.user_id = users.id
                        WHERE pastes.id = :id;",
                )?;
                let mut rows = stmt.query(named_params! {":id": id})?;
                match rows.next()? {
                    Some(row) => Ok(Some((
                        Paste::from_sql_row(row)?,
                        row.get::<usize, Username>(8)?,
                    ))),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_pair)
    }

    pub async fn update(
        mut self,
        db: &Database,
        filename: Option<String>,
        description: Option<String>,
        body: Option<String>,
    ) -> models::Result<()> {
        let original_filename = self.filename.clone();
        let original_body = self.body.clone();

        if let Some(filename) = filename {
            self.filename = Filename::try_from(filename)?;
        }
        if let Some(description) = description {
            self.description = Description::try_from(description)?;
        }
        if let Some(body) = body {
            self.body = Body::try_from(body)?;
        }

        let mut optional_html: Option<String> = None;
        let body_changed = original_body != self.body;
        let extension_changed = original_filename.extension() != self.filename.extension();
        if body_changed || extension_changed {
            optional_html =
                syntax_highlight::generate(self.body.as_ref(), self.filename.extension());
        }

        db.conn.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            {
                let mut pastes_stmt = tx.prepare(
                    r"UPDATE pastes
                    SET filename = :filename, description = :desc, body = :body, updated_at = unixepoch()
                    WHERE id = :id;"
                )?;
                pastes_stmt.execute(named_params! {":filename": self.filename, ":desc": self.description, ":body": self.body, ":id": self.id})?;

                if body_changed || extension_changed {
                    if let Some(html) = optional_html {
                        syntax_highlight::tx_cache_set(&tx, &self.id, &html)?;
                    } else {
                        syntax_highlight::tx_cache_expire(&tx, &self.id)?;
                    }
                }
            }
            tx.commit()?;
            Ok(())
        }).await?;
        Ok(())
    }

    pub async fn delete(self, db: &Database) -> models::Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare("DELETE FROM pastes WHERE id = :id;")?;
                let result = stmt.execute(named_params! {":id": self.id})?;
                Ok(result)
            })
            .await?;
        Ok(result)
    }
}

impl HasOrderedId for Paste {
    fn ordered_id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Display, Clone, AsRef, Into, Validate, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Filename {
    #[validate(
        length(min = 1, max = 256),
        custom(function = "Filename::validate_no_illegal_characters")
    )]
    inner: String,
}

impl Filename {
    pub fn new(s: &str) -> models::Result<Self> {
        let filename = Self {
            inner: s.trim().to_string(),
        };
        filename.validate()?;
        Ok(filename)
    }

    pub fn extension(&self) -> Option<&str> {
        if let Some((_, suffix)) = self.inner.rsplit_once('.') {
            Some(suffix)
        } else {
            None
        }
    }

    fn validate_no_illegal_characters(filename: &str) -> Result<(), validator::ValidationError> {
        if filename.contains(&['<', '>', ':', '"', '/', '\\', '|', '?', '*'][..]) {
            Err(validator::ValidationError::new(
                "filenames may not include the following characters: '<', '>', ':', '\"', '/', '\\', '|', '?', or '*'",
            ))
        } else if filename.ends_with('.') {
            Err(validator::ValidationError::new(
                "filenames may not end with a '.' character",
            ))
        } else {
            Ok(())
        }
    }
}

impl TryFrom<String> for Filename {
    type Error = models::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl std::str::FromStr for Filename {
    type Err = <Self as TryFrom<String>>::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Self as TryFrom<String>>::try_from(s.to_string())
    }
}

impl ToSql for Filename {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.inner.to_sql()
    }
}

impl FromSql for Filename {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value)
            .map(|s| Self::try_from(s).map_err(|e| FromSqlError::Other(Box::new(e))))?
    }
}

#[derive(Debug, Display, Clone, AsRef, Into, Validate, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Description {
    #[validate(length(max = 256))]
    inner: String,
}

impl Description {
    pub fn new(s: &str) -> models::Result<Self> {
        let description = Self {
            inner: s.trim().to_string(),
        };
        description.validate()?;
        Ok(description)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl TryFrom<String> for Description {
    type Error = models::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl std::str::FromStr for Description {
    type Err = <Self as TryFrom<String>>::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Self as TryFrom<String>>::try_from(s.to_string())
    }
}

impl ToSql for Description {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.inner.to_sql()
    }
}

impl FromSql for Description {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value)
            .map(|s| Self::try_from(s).map_err(|e| FromSqlError::Other(Box::new(e))))?
    }
}

#[derive(Debug, Display, Clone, AsRef, Into, PartialEq, Validate, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Body {
    #[validate(length(min = 1))]
    inner: String,
}

impl Body {
    pub fn new(s: &str) -> models::Result<Self> {
        let body = Self {
            inner: s.trim().to_string(),
        };
        body.validate()?;
        Ok(body)
    }
}

impl TryFrom<String> for Body {
    type Error = models::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl std::str::FromStr for Body {
    type Err = <Self as TryFrom<String>>::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Self as TryFrom<String>>::try_from(s.to_string())
    }
}

impl ToSql for Body {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.inner.to_sql()
    }
}

impl FromSql for Body {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value)
            .map(|s| Self::try_from(s).map_err(|e| FromSqlError::Other(Box::new(e))))?
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Visibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "secret")]
    Secret,
}

impl ToSql for Visibility {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        match self {
            Self::Public => Ok("public".into()),
            Self::Secret => Ok("secret".into()),
        }
    }
}

impl FromSql for Visibility {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).and_then(|as_string| match as_string.as_str() {
            "public" => Ok(Self::Public),
            "secret" => Ok(Self::Secret),
            _ => Err(FromSqlError::Other(
                "Unrecognized value for visibility".into(),
            )),
        })
    }
}

impl HasOrderedId for (Paste, Username) {
    fn ordered_id(&self) -> Uuid {
        self.0.id
    }
}
