use crate::db::Database;
use crate::models::prelude::*;
use crate::models::user::User;
use derive_more::From;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use secrecy::{ExposeSecret, Secret};
use sha2::{Digest, Sha256};
use tokio_rusqlite::named_params;

pub const API_KEY_HEADER_NAME: &str = "X-GLUESTICK-API-KEY";

pub struct ApiSession {
    pub api_key: HashedApiKey,
    pub user: User,
}

impl ApiSession {
    pub fn new(api_key: impl Into<HashedApiKey>, user: User) -> Self {
        Self {
            api_key: api_key.into(),
            user,
        }
    }

    pub async fn insert(self, db: &Database) -> Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "INSERT INTO api_sessions VALUES (:api_key, :user_id, unixepoch());",
                )?;
                let result = statement.execute(named_params! {
                    ":api_key": self.api_key,
                    ":user_id": self.user.id,
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }
}

#[derive(Clone)]
pub struct ApiKey(pub Secret<String>);

impl ApiKey {
    pub fn generate() -> Self {
        let mut rng = ChaCha20Rng::from_entropy();
        Self(Secret::new(format!("{:032x}", rng.gen::<u128>())))
    }
}

impl TryFrom<&str> for ApiKey {
    type Error = std::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        u128::from_str_radix(value, 16)?;
        Ok(Self(Secret::new(value.to_string())))
    }
}

impl ExposeSecret<String> for ApiKey {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[derive(From)]
pub struct HashedApiKey(Secret<Vec<u8>>);

impl From<&ApiKey> for HashedApiKey {
    fn from(key: &ApiKey) -> Self {
        let hash = Sha256::digest(key.expose_secret().as_bytes()).to_vec();
        Self(Secret::new(hash))
    }
}

impl ExposeSecret<Vec<u8>> for HashedApiKey {
    fn expose_secret(&self) -> &Vec<u8> {
        self.0.expose_secret()
    }
}

impl ToSql for HashedApiKey {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.expose_secret().to_sql()
    }
}

impl FromSql for HashedApiKey {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        Vec::<u8>::column_result(value).map(|vec| Ok(Secret::new(vec).into()))?
    }
}
