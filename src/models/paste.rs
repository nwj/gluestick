use crate::db::Database;
use crate::helpers::pagination::{Direction, HasOrderedId};
use crate::helpers::syntax_highlight;
use crate::models::prelude::*;
use crate::models::user::Username;
use derive_more::{AsRef, Display, IsVariant};
use jiff::Timestamp;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Type, ValueRef};
use rusqlite::{named_params, Row, Transaction, TransactionBehavior};
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct Paste {
    pub id: Uuid,
    pub user_id: Uuid,
    pub filename: Filename,
    pub description: Description,
    pub body: Body,
    pub visibility: Visibility,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Paste {
    pub fn new(
        user_id: Uuid,
        filename: Filename,
        description: Description,
        body: Body,
        visibility: Visibility,
    ) -> Result<Self> {
        let now = Timestamp::now();
        Ok(Self {
            id: Uuid::now_v7(),
            user_id,
            filename,
            description,
            body,
            visibility,
            created_at: now,
            updated_at: now,
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
            created_at: Timestamp::from_millisecond(row.get(6)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(6, Type::Integer, Box::new(e))
            })?,
            updated_at: Timestamp::from_millisecond(row.get(7)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(7, Type::Integer, Box::new(e))
            })?,
        })
    }

    pub async fn syntax_highlight(&self, db: &Database) -> Result<Option<String>> {
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
    ) -> Result<Vec<Paste>> {
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
                    FROM pastes
                    WHERE visibility = 'public' {cursor_sql}
                    ORDER BY pastes.id {direction_sql}
                    LIMIT :limit;"
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
    ) -> Result<Vec<(Paste, Username)>> {
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

    pub async fn cursor_paginated_for_user_id(
        db: &Database,
        user_id: Uuid,
        limit: usize,
        direction: Direction,
        cursor: Option<Uuid>,
    ) -> Result<Vec<Paste>> {
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
                    FROM pastes
                    WHERE user_id = :user_id AND visibility = 'public' {cursor_sql}
                    ORDER BY pastes.id {direction_sql}
                    LIMIT :limit;"
                );
                let mut stmt = conn.prepare(&raw_sql)?;
                match cursor {
                    None => {
                        let paste_iter = stmt.query_map(named_params! {":user_id": user_id, ":limit": limit}, Paste::from_sql_row)?;
                        Ok(paste_iter.collect::<Result<Vec<_>, _>>()?)
                    }
                    Some(cursor) => {
                        let paste_iter = stmt.query_map(named_params! {":user_id": user_id, ":limit": limit, ":cursor": cursor}, Paste::from_sql_row)?;
                        Ok(paste_iter.collect::<Result<Vec<_>, _>>()?)
                    }
                }
            })
            .await?;
        Ok(pastes)
    }

    pub async fn cursor_paginated_for_user_id_with_secrets(
        db: &Database,
        user_id: Uuid,
        limit: usize,
        direction: Direction,
        cursor: Option<Uuid>,
    ) -> Result<Vec<Paste>> {
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
                    FROM pastes
                    WHERE user_id = :user_id {cursor_sql}
                    ORDER BY pastes.id {direction_sql}
                    LIMIT :limit;"
                );
                let mut stmt = conn.prepare(&raw_sql)?;
                match cursor {
                    None => {
                        let paste_iter = stmt.query_map(named_params! {":user_id": user_id, ":limit": limit}, Paste::from_sql_row)?;
                        Ok(paste_iter.collect::<Result<Vec<_>, _>>()?)
                    }
                    Some(cursor) => {
                        let paste_iter = stmt.query_map(named_params! {":user_id": user_id, ":limit": limit, ":cursor": cursor}, Paste::from_sql_row)?;
                        Ok(paste_iter.collect::<Result<Vec<_>, _>>()?)
                    }
                }
            })
            .await?;
        Ok(pastes)
    }

    pub async fn insert(self, db: &Database) -> Result<()> {
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
                            ":created_at": self.created_at.as_millisecond(),
                            ":updated_at": self.updated_at.as_millisecond(),
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

    pub async fn find(db: &Database, id: Uuid) -> Result<Option<Paste>> {
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

    pub async fn find_scoped_by_user_id(
        db: &Database,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Paste>> {
        let optional_paste = db
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    r"SELECT id, user_id, filename, description, body, visibility, created_at, updated_at
                    FROM pastes
                    WHERE id = :id AND user_id = :user_id;",
                )?;
                let mut rows = stmt.query(named_params! {":id": id, ":user_id": user_id})?;
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

    pub async fn update(
        mut self,
        db: &Database,
        filename: Option<Filename>,
        description: Option<Description>,
        body: Option<Body>,
    ) -> Result<()> {
        let original_filename = self.filename.clone();
        let original_body = self.body.clone();

        if let Some(filename) = filename {
            self.filename = filename;
        }
        if let Some(description) = description {
            self.description = description;
        }
        if let Some(body) = body {
            self.body = body;
        }

        let mut maybe_html: Option<String> = None;
        let body_changed = original_body != self.body;
        let extension_changed = original_filename.extension() != self.filename.extension();
        if body_changed || extension_changed {
            maybe_html = syntax_highlight::generate(self.body.as_ref(), self.filename.extension());
        }

        db.conn.call(move |conn| {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            {
                let mut pastes_stmt = tx.prepare(
                    r"UPDATE pastes
                    SET filename = :filename, description = :desc, body = :body, updated_at = :updated_at
                    WHERE id = :id;"
                )?;
                pastes_stmt.execute(named_params! {
                    ":filename": self.filename,
                    ":desc": self.description,
                    ":body": self.body,
                    ":updated_at": Timestamp::now().as_millisecond(),
                    ":id": self.id,
                })?;

                if body_changed || extension_changed {
                    if let Some(html) = maybe_html {
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

    pub async fn delete(self, db: &Database) -> Result<usize> {
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

#[derive(Clone, Debug, Display, Serialize)]
#[serde(transparent)]
pub struct Filename(String);

impl Filename {
    pub fn new(s: &str) -> Result<Self> {
        let filename = Self(s.trim().to_string());
        Ok(filename)
    }

    pub fn extension(&self) -> Option<&str> {
        if let Some((_, suffix)) = self.0.rsplit_once('.') {
            Some(suffix)
        } else {
            None
        }
    }
}

impl FromStr for Filename {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Err(Error::Parse("Filename may not be blank".into()))
        } else if s.chars().count() > 256 {
            Err(Error::Parse(
                "Filename may not be longer than 256 characters".into(),
            ))
        } else if s
            .chars()
            .any(|c| ['<', '>', ':', '"', '/', '\\', '|', '?', '*'].contains(&c))
        {
            Err(Error::Parse(
                "Filename may not contain the following characters: < > : \" / \\ | ? *".into(),
            ))
        } else if s.ends_with('.') {
            Err(Error::Parse(
                "Filename may not end with a '.' character".into(),
            ))
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl TryFrom<&String> for Filename {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl ToSql for Filename {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.0.to_sql()
    }
}

impl FromSql for Filename {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(Self)
    }
}

#[derive(Clone, Debug, Display, Serialize)]
#[serde(transparent)]
pub struct Description(String);

impl Description {
    pub fn new(s: &str) -> Result<Self> {
        let description = Self(s.trim().to_string());
        Ok(description)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl FromStr for Description {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().count() > 256 {
            Err(Error::Parse(
                "Description may not be longer than 256 characters".into(),
            ))
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl TryFrom<&String> for Description {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl ToSql for Description {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.0.to_sql()
    }
}

impl FromSql for Description {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(Self)
    }
}

#[derive(AsRef, Clone, Debug, Display, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Body(String);

impl Body {
    pub fn new(s: &str) -> Result<Self> {
        let body = Self(s.trim().to_string());
        Ok(body)
    }
}

impl FromStr for Body {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Err(Error::Parse("Body may not be blank".into()))
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

impl TryFrom<&String> for Body {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl ToSql for Body {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.0.to_sql()
    }
}

impl FromSql for Body {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(Self)
    }
}

#[derive(Clone, Copy, Debug, IsVariant, Serialize)]
pub enum Visibility {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "secret")]
    Secret,
}

impl FromStr for Visibility {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "public" => Ok(Self::Public),
            "secret" => Ok(Self::Secret),
            _ => Err(Error::Parse(
                "Unrecognized value for visibility. Valid values are 'public' or 'secret'".into(),
            )),
        }
    }
}

impl TryFrom<&String> for Visibility {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.parse()
    }
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
        String::column_result(value).and_then(|s| {
            Self::try_from(&s)
                .map_err(|_| FromSqlError::Other("Unrecognized value for visibility".into()))
        })
    }
}

impl HasOrderedId for (Paste, Username) {
    fn ordered_id(&self) -> Uuid {
        self.0.id
    }
}
