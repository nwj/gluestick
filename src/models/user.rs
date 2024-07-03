use crate::db::Database;
use crate::models::api_session::HashedApiKey;
use crate::models::prelude::*;
use crate::models::session::HashedSessionToken;
use crate::params::users::CreateUserParams;
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use derive_more::Display;
use rand::rngs::OsRng;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{named_params, Row};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: Username,
    pub email: EmailAddress,
    pub password: HashedPassword,
}

impl User {
    fn new(
        username: impl Into<String>,
        email: impl Into<String>,
        password: Secret<String>,
    ) -> Result<Self> {
        let username = username.into();
        let email = email.into();
        Ok(User {
            id: Uuid::now_v7(),
            username: Username::new(username),
            email: EmailAddress::new(email),
            password: HashedPassword::new(password)?,
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

    pub async fn find_by_username(db: &Database, username: String) -> Result<Option<User>> {
        let optional_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "SELECT id, username, email, password FROM users WHERE username = :username;",
                )?;
                let mut rows = statement.query(named_params! {":username": username})?;
                match rows.next()? {
                    Some(row) => Ok(Some(User::from_sql_row(row)?)),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_user)
    }

    pub async fn find_by_session_token(
        db: &Database,
        token: impl Into<HashedSessionToken>,
    ) -> Result<Option<User>> {
        let token = token.into();
        let optional_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    r"SELECT users.id, users.username, users.email, users.password
                    FROM users JOIN sessions ON users.id = sessions.user_id
                    WHERE sessions.session_token = :token;",
                )?;
                let mut rows = statement.query(named_params! {":token": token})?;
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

    pub async fn find_by_api_key(
        db: &Database,
        key: impl Into<HashedApiKey>,
    ) -> Result<Option<User>> {
        let key = key.into();
        let optional_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    r"SELECT users.id, users.username, users.email, users.password
                    FROM users JOIN api_sessions ON users.id = api_sessions.user_id
                    WHERE api_sessions.api_key = :key;",
                )?;
                let mut rows = statement.query(named_params! {":key": key})?;
                match rows.next()? {
                    Some(row) => Ok(Some(User::from_sql_row(row)?)),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_user)
    }
}

impl TryFrom<CreateUserParams> for User {
    type Error = Error;

    fn try_from(params: CreateUserParams) -> std::result::Result<Self, Self::Error> {
        let username = params.username.into_inner();
        let email = params.email.into_inner();
        let password = params.password.into_inner();

        Self::new(username, email, password)
    }
}

#[derive(Clone, Debug, Display)]
pub struct Username(String);

impl Username {
    fn new(username: impl Into<String>) -> Self {
        let username = username.into();
        Self(username)
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

#[derive(Clone, Debug, Display)]
pub struct EmailAddress(String);

impl EmailAddress {
    fn new(email: impl Into<String>) -> Self {
        let email = email.into();
        Self(email.to_lowercase())
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
    pub fn new(password: Secret<String>) -> Result<Self, argon2::password_hash::Error> {
        Ok(HashedPassword(Secret::new(
            Argon2::default()
                .hash_password(
                    password.expose_secret().as_bytes(),
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
