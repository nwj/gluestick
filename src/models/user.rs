use crate::{
    db::Database,
    models,
    models::{api_session::ApiKey, session::SessionToken},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use rusqlite::named_params;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Clone, Deserialize, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: Secret<String>,
}

impl User {
    pub fn new(username: String, email: String, password: Secret<String>) -> models::Result<Self> {
        let id = Uuid::now_v7();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password = Secret::new(
            argon2
                .hash_password(password.expose_secret().as_bytes(), &salt)?
                .to_string(),
        );

        Ok(User {
            id,
            username,
            email,
            password,
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
                    ":username": self.username.to_lowercase(),
                    ":email": self.email.to_lowercase(),
                    ":password": self.password.expose_secret()
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub async fn find_by_email(db: &Database, email: String) -> models::Result<Option<User>> {
        let optional_user = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "SELECT id, username, email, password FROM users WHERE email = :email;",
                )?;
                let mut rows = statement.query(named_params! {":email": email})?;
                match rows.next()? {
                    Some(row) => Ok(Some(
                        serde_rusqlite::from_row(row)
                            .map_err(|e| tokio_rusqlite::Error::Other(Box::new(e)))?,
                    )),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_user)
    }

    pub async fn find_by_session_token(
        db: &Database,
        token: SessionToken,
    ) -> models::Result<Option<User>> {
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
                    Some(row) => Ok(Some(
                        serde_rusqlite::from_row(row)
                            .map_err(|e| tokio_rusqlite::Error::Other(Box::new(e)))?,
                    )),
                    None => Ok(None),
                }
            })
            .await?;

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
                    Some(row) => Ok(Some(
                        serde_rusqlite::from_row(row)
                            .map_err(|e| tokio_rusqlite::Error::Other(Box::new(e)))?,
                    )),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_user)
    }
}
