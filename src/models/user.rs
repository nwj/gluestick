use crate::{
    db::Database,
    models,
    models::{api_session::ApiKey, session::SessionToken},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use rusqlite::{
    named_params,
    types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
    Row,
};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
    pub password: HashedPassword,
}

impl User {
    pub fn new(username: String, email: String, password: Password) -> models::Result<Self> {
        Ok(User {
            id: Uuid::now_v7(),
            username: Username::parse(&username)?,
            email: EmailAddress::parse(&email)?,
            password: password.to_hash()?,
        })
    }

    pub fn from_sql_row(row: &Row) -> models::Result<Self> {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            email: row.get(2)?,
            password: row.get(3)?,
        })
    }

    pub async fn insert(self, db: &Database) -> models::Result<usize> {
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

    pub async fn find_by_email(db: &Database, email: String) -> models::Result<Option<User>> {
        let optional_result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "SELECT id, username, email, password FROM users WHERE email = :email;",
                )?;
                let mut rows = statement.query(named_params! {":email": email})?;
                match rows.next()? {
                    Some(row) => Ok(Some(User::from_sql_row(row))),
                    None => Ok(None),
                }
            })
            .await?;

        let optional_user = optional_result.transpose()?;
        Ok(optional_user)
    }

    pub async fn find_by_session_token(
        db: &Database,
        token: SessionToken,
    ) -> models::Result<Option<User>> {
        let optional_result = db
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
                    Some(row) => Ok(Some(User::from_sql_row(row))),
                    None => Ok(None),
                }
            })
            .await?;

        let optional_user = optional_result.transpose()?;
        Ok(optional_user)
    }

    pub async fn delete_sessions(self, db: &Database) -> models::Result<usize> {
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

    pub async fn find_by_api_key(db: &Database, key: ApiKey) -> models::Result<Option<User>> {
        let optional_result = db
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
                    Some(row) => Ok(Some(User::from_sql_row(row))),
                    None => Ok(None),
                }
            })
            .await?;

        let optional_user = optional_result.transpose()?;
        Ok(optional_user)
    }
}

#[derive(Debug, Clone, Validate)]
pub struct Username {
    #[validate(length(min = 3, max = 32), custom(function = "validate_alphanumeric"))]
    private: String,
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

impl Username {
    pub fn parse(s: &str) -> models::Result<Self> {
        let username = Self {
            private: s.to_string(),
        };
        username.validate()?;
        Ok(username)
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.private
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.private)
    }
}

impl ToSql for Username {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.private.to_sql()
    }
}

impl FromSql for Username {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(|as_string| Ok(Self { private: as_string }))?
    }
}

#[derive(Debug, Clone, Validate)]
pub struct EmailAddress {
    #[validate(email)]
    private: String,
}

impl EmailAddress {
    pub fn parse(s: &str) -> models::Result<Self> {
        let email = Self {
            private: s.to_string(),
        };
        email.validate()?;
        Ok(email)
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.private
    }
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.private)
    }
}

impl ToSql for EmailAddress {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.private.to_sql()
    }
}

impl FromSql for EmailAddress {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(|as_string| Ok(Self { private: as_string }))?
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Password(Secret<String>);

impl Password {
    pub fn new(password: Secret<String>) -> Self {
        Self(password)
    }

    pub fn to_hash(self) -> models::Result<HashedPassword> {
        Ok(HashedPassword(Self::hash_password(self.0)?))
    }

    fn hash_password(password: Secret<String>) -> models::Result<Secret<String>> {
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

impl ExposeSecret<String> for Password {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[derive(Debug, Clone)]
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
        String::column_result(value).map(|as_string| Ok(HashedPassword(Secret::new(as_string))))?
    }
}
