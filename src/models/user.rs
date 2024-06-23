use crate::db::Database;
use crate::models::api_session::ApiKey;
use crate::models::prelude::*;
use crate::models::session::SessionToken;
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use derive_more::{AsRef, Display, From, Into};
use rand::rngs::OsRng;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{named_params, Row};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
    pub password: HashedPassword,
}

impl User {
    pub fn new(username: String, email: String, password: &Password) -> Result<Self> {
        Ok(User {
            id: Uuid::now_v7(),
            username: Username::try_from(username)?,
            email: EmailAddress::try_from(email)?,
            password: password.to_hash()?,
        })
    }

    pub fn from_sql_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            email: row.get(2)?,
            password: row.get(3)?,
        })
    }

    pub fn verify_password(&self, password: &Password) -> Result<()> {
        Argon2::default().verify_password(
            password.expose_secret().as_bytes(),
            &PasswordHash::new(self.password.expose_secret())?,
        )?;
        Ok(())
    }

    pub async fn insert(self, db: &Database) -> Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("INSERT INTO users VALUES (:id, :username, :email, :password);")?;
                let result = statement.execute(named_params! {
                    ":id": self.id,
                    ":username": self.username,
                    ":email": self.email,
                    ":password": self.password.expose_secret()
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub async fn find_by_email(db: &Database, email: String) -> Result<Option<User>> {
        let optional_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "SELECT id, username, email, password FROM users WHERE email = :email;",
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

    pub async fn find_by_session_token(db: &Database, token: SessionToken) -> Result<Option<User>> {
        let optional_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    r"SELECT users.id, users.username, users.email, users.password
                    FROM users JOIN sessions ON users.id = sessions.user_id
                    WHERE sessions.session_token = :token;",
                )?;
                let mut rows =
                    statement.query(named_params! {":token": token.to_hash().expose_secret()})?;
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
                    conn.prepare("DELETE FROM sessions WHERE user_id = :user_id;")?;
                let result = statement.execute(named_params! {
                    ":user_id": self.id
                })?;
                Ok(result)
            })
            .await?;
        Ok(result)
    }

    pub async fn find_by_api_key(db: &Database, key: ApiKey) -> Result<Option<User>> {
        let optional_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    r"SELECT users.id, users.username, users.email, users.password
                    FROM users JOIN api_sessions ON users.id = api_sessions.user_id
                    WHERE api_sessions.api_key = :key;",
                )?;
                let mut rows =
                    statement.query(named_params! {":key": key.to_hash().expose_secret()})?;
                match rows.next()? {
                    Some(row) => Ok(Some(User::from_sql_row(row)?)),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_user)
    }
}

#[derive(AsRef, Clone, Debug, Display, Into, Validate)]
pub struct Username {
    #[validate(
        length(min = 3, max = 32),
        custom(function = "Username::validate_alphanumeric")
    )]
    inner: String,
}

impl Username {
    pub fn new(s: String) -> Result<Self> {
        let username = Self { inner: s };
        username.validate()?;
        Ok(username)
    }

    fn validate_alphanumeric(username: &str) -> Result<(), validator::ValidationError> {
        if username.chars().all(char::is_alphanumeric) {
            Ok(())
        } else {
            Err(validator::ValidationError::new(
                "username must be alphanumeric",
            ))
        }
    }
}

impl TryFrom<String> for Username {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::str::FromStr for Username {
    type Err = <Self as TryFrom<String>>::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Self as TryFrom<String>>::try_from(s.to_string())
    }
}

impl ToSql for Username {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.inner.to_sql()
    }
}

impl FromSql for Username {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value)
            .map(|s| Self::try_from(s).map_err(|e| FromSqlError::Other(Box::new(e))))?
    }
}

#[derive(AsRef, Clone, Debug, Display, Into, Validate)]
pub struct EmailAddress {
    #[validate(email)]
    inner: String,
}

impl EmailAddress {
    pub fn new(s: &str) -> Result<Self> {
        let email = Self {
            inner: s.to_lowercase(),
        };
        email.validate()?;
        Ok(email)
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl std::str::FromStr for EmailAddress {
    type Err = <Self as TryFrom<String>>::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <Self as TryFrom<String>>::try_from(s.to_string())
    }
}

impl ToSql for EmailAddress {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.inner.to_sql()
    }
}

impl FromSql for EmailAddress {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value)
            .map(|s| Self::try_from(s).map_err(|e| FromSqlError::Other(Box::new(e))))?
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Password(Secret<String>);

impl Password {
    pub fn to_hash(&self) -> Result<HashedPassword> {
        Ok(HashedPassword(Self::hash_password(&self.0)?))
    }

    fn hash_password(password: &Secret<String>) -> Result<Secret<String>> {
        Ok(Secret::new(
            Argon2::default()
                .hash_password(
                    password.expose_secret().as_bytes(),
                    &SaltString::generate(&mut OsRng),
                )?
                .to_string(),
        ))
    }
}

impl Validate for Password {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if self.expose_secret().len() < 8 {
            errors.add(
                "0",
                validator::ValidationError::new("password must be at least 8 characters long"),
            );
        } else if self.expose_secret().len() > 256 {
            errors.add(
                "0",
                validator::ValidationError::new("password may not be longer than 256 characters"),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl ExposeSecret<String> for Password {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[derive(Clone, Debug, From)]
pub struct HashedPassword(Secret<String>);

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
        String::column_result(value).map(|string| Ok(Secret::new(string).into()))?
    }
}
