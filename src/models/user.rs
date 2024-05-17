use crate::{db::Database, models::session::SessionToken};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use rusqlite::named_params;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password: Secret<String>,
}

impl User {
    pub async fn insert(
        db: &Database,
        username: String,
        email: String,
        password: Secret<String>,
    ) -> Result<usize, Error> {
        let id = Uuid::now_v7();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = Secret::new(
            argon2
                .hash_password(password.expose_secret().as_bytes(), &salt)?
                .to_string(),
        );

        let result = db
            .conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("INSERT INTO users VALUES (:id, :username, :email, :password);")?;
                let result = statement.execute(named_params! {
                    ":id": id,
                    ":username": username.to_lowercase(),
                    ":email": email.to_lowercase(),
                    ":password": password_hash.expose_secret()
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub async fn find_by_email(
        db: &Database,
        email: String,
    ) -> Result<Option<User>, tokio_rusqlite::Error> {
        let maybe_user = db
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

        Ok(maybe_user)
    }

    pub async fn find_by_session_token(
        db: &Database,
        token: SessionToken,
    ) -> Result<Option<User>, tokio_rusqlite::Error> {
        let maybe_user = db
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

        Ok(maybe_user)
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] tokio_rusqlite::Error),
    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),
}
