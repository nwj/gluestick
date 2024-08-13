use crate::db::Database;
use crate::models::prelude::*;
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use derive_more::Display;
use jiff::Timestamp;
use rand::rngs::OsRng;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, Type, ValueRef};
use rusqlite::{named_params, Row};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
    pub password: HashedPassword,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl User {
    pub fn new(
        username: impl Into<String>,
        email: impl Into<String>,
        password: impl Into<Secret<String>>,
    ) -> Result<Self> {
        let now = Timestamp::now();
        Ok(User {
            id: Uuid::now_v7(),
            username: Username::new(username),
            email: EmailAddress::new(email),
            password: HashedPassword::new(password)?,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn from_sql_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            email: row.get(2)?,
            password: row.get(3)?,
            created_at: Timestamp::from_millisecond(row.get(4)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(4, Type::Integer, Box::new(e))
            })?,
            updated_at: Timestamp::from_millisecond(row.get(5)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(5, Type::Integer, Box::new(e))
            })?,
        })
    }

    pub fn verify_password(&self, password: &String) -> Result<()> {
        Argon2::default().verify_password(
            password.as_bytes(),
            &PasswordHash::new(self.password.expose_secret())?,
        )?;
        Ok(())
    }

    pub async fn insert(self, db: &Database) -> Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("INSERT INTO users VALUES (:id, :username, :email, :password, :created_at, :updated_at);")?;
                let result = statement.execute(named_params! {
                    ":id": self.id,
                    ":username": self.username,
                    ":email": self.email,
                    ":password": self.password.expose_secret(),
                    ":created_at": self.created_at.as_millisecond(),
                    ":updated_at": self.updated_at.as_millisecond(),
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub async fn update_password(
        &self,
        db: &Database,
        new_password: impl Into<Secret<String>>,
    ) -> Result<usize> {
        let id = self.id;
        let hashed_password = HashedPassword::new(new_password)?;
        let result = db
            .conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("UPDATE users SET password = :password, updated_at = :updated_at WHERE id = :id;")?;
                let result = statement.execute(named_params! {
                    ":password": hashed_password,
                    ":id": id,
                    ":updated_at": Timestamp::now().as_millisecond(),
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub async fn find_by_email(db: &Database, email: impl Into<String>) -> Result<Option<User>> {
        let email = email.into();
        let optional_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "SELECT id, username, email, password, created_at, updated_at FROM users WHERE email = :email;",
                )?;
                let mut rows = statement.query(named_params! {":email": email.to_lowercase()})?;
                match rows.next()? {
                    Some(row) => Ok(Some(User::from_sql_row(row)?)),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_user)
    }

    pub async fn find_by_username(
        db: &Database,
        username: impl Into<String>,
    ) -> Result<Option<User>> {
        let username = username.into();
        let optional_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "SELECT id, username, email, password, created_at, updated_at FROM users WHERE username = :username;",
                )?;
                let mut rows =
                    statement.query(named_params! {":username": username.to_lowercase()})?;
                match rows.next()? {
                    Some(row) => Ok(Some(User::from_sql_row(row)?)),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_user)
    }

    pub async fn delete_sessions(self, db: &Database) -> Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("DELETE FROM session_tokens WHERE user_id = :user_id;")?;
                let result = statement.execute(named_params! {
                    ":user_id": self.id
                })?;
                Ok(result)
            })
            .await?;
        Ok(result)
    }
}

#[derive(Clone, Debug, Display, PartialEq)]
pub struct Username(String);

impl Username {
    fn new(username: impl Into<String>) -> Self {
        Self(username.into().to_lowercase())
    }
}

impl ToSql for Username {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.0.to_sql()
    }
}

impl FromSql for Username {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(|string| Ok(Self(string)))?
    }
}

#[derive(Clone, Debug, Display, PartialEq)]
pub struct EmailAddress(String);

impl EmailAddress {
    fn new(email: impl Into<String>) -> Self {
        Self(email.into().to_lowercase())
    }
}

impl ToSql for EmailAddress {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.0.to_sql()
    }
}

impl FromSql for EmailAddress {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(|string| Ok(Self(string)))?
    }
}

#[derive(Clone, Debug)]
pub struct HashedPassword(Secret<String>);

impl HashedPassword {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(password: impl Into<Secret<String>>) -> Result<Self, argon2::password_hash::Error> {
        Ok(HashedPassword(Secret::new(
            Argon2::default()
                .hash_password(
                    password.into().expose_secret().as_bytes(),
                    &SaltString::generate(&mut OsRng),
                )?
                .to_string(),
        )))
    }
}

impl ExposeSecret<String> for HashedPassword {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

impl ToSql for HashedPassword {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.expose_secret().to_sql()
    }
}

impl FromSql for HashedPassword {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(|string| Ok(Self(Secret::new(string))))?
    }
}

impl PartialEq for HashedPassword {
    fn eq(&self, other: &HashedPassword) -> bool {
        self.expose_secret() == other.expose_secret()
    }
}
