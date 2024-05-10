use crate::db;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use rusqlite::named_params;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password: Secret<String>,
}

impl User {
    pub async fn insert(
        db: &db::Database,
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
                    conn.prepare("INSERT INTO users VALUES (:id, :email, :username, :password);")?;
                let result = statement.execute(named_params! {
                    ":id": id,
                    ":email": email.to_lowercase(),
                    ":username": username.to_lowercase(),
                    ":password": password_hash.expose_secret()
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
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