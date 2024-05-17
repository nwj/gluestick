use crate::db::Database;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use secrecy::{ExposeSecret, Secret};
use sha2::{Digest, Sha256};
use std::num::ParseIntError;
use tokio_rusqlite::named_params;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionToken(Secret<String>);

impl SessionToken {
    pub fn generate() -> Self {
        // The OWASP checklist for session tokens:
        // - has a size of at least 128-bits: ours is 128-bits
        // - contains at least 64-bits of entropy: use of ChaCha20 seeded by the OS should ensure this
        // - must be unique: uniqueness is statistically likely here, but enforced elsewhere by database constraint
        //
        // See: https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
        let mut rng = ChaCha20Rng::from_entropy();
        SessionToken(Secret::new(format!("{:032x}", rng.gen::<u128>())))
    }

    pub fn to_hash(&self) -> HashedSessionToken {
        HashedSessionToken(Secret::new(
            Sha256::digest(self.expose_secret().as_bytes()).to_vec(),
        ))
    }

    pub fn parse(s: impl AsRef<str>) -> Result<Self, ParseIntError> {
        let s = s.as_ref();
        u128::from_str_radix(s, 16)?;
        Ok(SessionToken(Secret::new(s.to_string())))
    }

    pub async fn insert(
        self,
        db: &Database,
        user_id: Uuid,
    ) -> Result<usize, tokio_rusqlite::Error> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("INSERT INTO sessions VALUES (:session_token, :user_id);")?;
                let result = statement.execute(named_params! {
                    ":session_token": self.to_hash().expose_secret(),
                    ":user_id": user_id,
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub async fn delete_by_user_id(
        db: &Database,
        user_id: Uuid,
    ) -> Result<usize, tokio_rusqlite::Error> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("DELETE FROM sessions WHERE user_id = :user_id;")?;
                let result = statement.execute(named_params! {
                    ":user_id": user_id
                })?;
                Ok(result)
            })
            .await?;
        Ok(result)
    }
}

impl ExposeSecret<String> for SessionToken {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

pub struct HashedSessionToken(Secret<Vec<u8>>);

impl ExposeSecret<Vec<u8>> for HashedSessionToken {
    fn expose_secret(&self) -> &Vec<u8> {
        self.0.expose_secret()
    }
}
