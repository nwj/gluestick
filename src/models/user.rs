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
use std::convert::TryFrom;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Display)]
#[display("{{ id: {id}, username: {username}, email: {email} }}")]
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
        username: Username,
        email: EmailAddress,
        password: UnhashedPassword,
    ) -> Result<Self> {
        let now = Timestamp::now();
        let hashed_password = HashedPassword::try_from(password)?;
        Ok(User {
            id: Uuid::now_v7(),
            username,
            email,
            password: hashed_password,
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

    pub fn verify_password(&self, password: &UnhashedPassword) -> Result<()> {
        Ok(Argon2::default().verify_password(
            password.expose_secret().as_bytes(),
            &PasswordHash::new(self.password.expose_secret())?,
        )?)
    }

    pub async fn insert(self, db: &Database) -> Result<usize> {
        tracing::info!("inserting user {self}");
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
        new_password: UnhashedPassword,
    ) -> Result<usize> {
        tracing::info!("updating password for user {self}");
        let id = self.id;
        let hashed_password = HashedPassword::try_from(new_password)?;
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

    pub async fn find_by_email(db: &Database, email: EmailAddress) -> Result<Option<User>> {
        let maybe_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "SELECT id, username, email, password, created_at, updated_at FROM users WHERE email = :email;",
                )?;
                let mut rows = statement.query(named_params! {":email": email})?;
                match rows.next()? {
                    Some(row) => Ok(Some(User::from_sql_row(row)?)),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(maybe_user)
    }

    pub async fn find_by_username(db: &Database, username: Username) -> Result<Option<User>> {
        let maybe_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "SELECT id, username, email, password, created_at, updated_at FROM users WHERE username = :username;",
                )?;
                let mut rows =
                    statement.query(named_params! {":username": username})?;
                match rows.next()? {
                    Some(row) => Ok(Some(User::from_sql_row(row)?)),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(maybe_user)
    }

    pub async fn delete_sessions(self, db: &Database) -> Result<usize> {
        tracing::info!("deleting all sessions for user {self}");
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

impl FromStr for Username {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        if s.trim().is_empty() {
            Err(Error::Parse("Username may not be blank".into()))
        } else if s.chars().count() > 32 {
            Err(Error::Parse(
                "Username is too long (maximum is 32 characters)".into(),
            ))
        } else if !s.chars().all(|c| c.is_alphanumeric() || c == '-')
            || s.contains("--")
            || s.starts_with('-')
            || s.ends_with('-')
        {
            Err(Error::Parse(
                "Username may only contain alphanumeric characters or single hyphens, and may not begin or end with a hyphen".into(),
            ))
        } else if s == "api"
            || s == "api_sessions"
            || s == "assets"
            || s == "health"
            || s == "login"
            || s == "logout"
            || s == "new"
            || s == "pastes"
            || s == "settings"
            || s == "signup"
        {
            Err(Error::Parse("Username is unavailable".into()))
        } else {
            Ok(Self(s))
        }
    }
}

impl TryFrom<&String> for Username {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl ToSql for Username {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.0.to_sql()
    }
}

impl FromSql for Username {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(Self)
    }
}

#[derive(Clone, Debug, Display, PartialEq)]
pub struct EmailAddress(String);

impl FromStr for EmailAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        if s.trim().is_empty() {
            Err(Error::Parse("Email may not be blank".into()))
        } else if !s.contains('@') {
            Err(Error::Parse("Email is missing the '@' symbol".into()))
        } else if s.starts_with('@') {
            Err(Error::Parse(
                "Email is missing the username part before the '@' symbol".into(),
            ))
        } else if s.ends_with('@') {
            Err(Error::Parse(
                "Email is missing the domain part after the '@' symbol".into(),
            ))
        } else {
            Ok(Self(s))
        }
    }
}

impl TryFrom<&String> for EmailAddress {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl ToSql for EmailAddress {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.0.to_sql()
    }
}

impl FromSql for EmailAddress {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(Self)
    }
}

#[derive(Clone, Debug)]
pub struct UnhashedPassword(Secret<String>);

impl TryFrom<Secret<String>> for UnhashedPassword {
    type Error = Error;

    fn try_from(value: Secret<String>) -> Result<Self, Self::Error> {
        if value.expose_secret().is_empty() {
            Err(Error::Parse("Password may not be blank".into()))
        } else if value.expose_secret().chars().count() < 8 {
            Err(Error::Parse(
                "Password is too short (minimum is 8 characters)".into(),
            ))
        } else if value.expose_secret().chars().count() > 256 {
            Err(Error::Parse(
                "Password is too long (maximum is 256 characters)".into(),
            ))
        } else {
            Ok(Self(value))
        }
    }
}

impl ExposeSecret<String> for UnhashedPassword {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

impl PartialEq for UnhashedPassword {
    fn eq(&self, other: &UnhashedPassword) -> bool {
        self.expose_secret() == other.expose_secret()
    }
}

#[derive(Clone, Debug)]
pub struct HashedPassword(Secret<String>);

impl TryFrom<UnhashedPassword> for HashedPassword {
    type Error = Error;

    fn try_from(value: UnhashedPassword) -> Result<Self, Self::Error> {
        Ok(HashedPassword(Secret::new(
            Argon2::default()
                .hash_password(
                    value.expose_secret().as_bytes(),
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
        String::column_result(value).map(|s| Self(Secret::new(s)))
    }
}

impl PartialEq for HashedPassword {
    fn eq(&self, other: &HashedPassword) -> bool {
        self.expose_secret() == other.expose_secret()
    }
}
