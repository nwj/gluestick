use crate::db::Database;
use crate::models::prelude::*;
use crate::models::user::User;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use secrecy::{ExposeSecret, Secret};
use sha2::{Digest, Sha256};
use tokio_rusqlite::named_params;

pub struct ApiSession {
    pub api_key: HashedApiKey,
    pub user: User,
}

impl ApiSession {
    pub fn new(api_key: &ApiKey, user: User) -> Self {
        Self {
            api_key: api_key.to_hash(),
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
                    ":api_key": self.api_key.expose_secret(),
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

    pub fn parse(s: impl AsRef<str>) -> Result<Self> {
        let s = s.as_ref();
        u128::from_str_radix(s, 16)?;
        Ok(Self(Secret::new(s.to_string())))
    }

    pub fn to_hash(&self) -> HashedApiKey {
        HashedApiKey(Secret::new(
            Sha256::digest(self.expose_secret().as_bytes()).to_vec(),
        ))
    }
}

impl ExposeSecret<String> for ApiKey {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

pub struct HashedApiKey(Secret<Vec<u8>>);

impl ExposeSecret<Vec<u8>> for HashedApiKey {
    fn expose_secret(&self) -> &Vec<u8> {
        self.0.expose_secret()
    }
}
