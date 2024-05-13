use crate::db::Database;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use secrecy::{ExposeSecret, Secret};
use sha2::{Digest, Sha256};
use tokio_rusqlite::named_params;
use uuid::Uuid;

#[allow(dead_code)]
pub struct Session {
    session_token: Secret<Vec<u8>>,
    user_id: Uuid,
}

impl Session {
    fn generate_session_token() -> Secret<String> {
        // The OWASP checklist for session tokens:
        // - should have a size of at least 128-bits: ours is 128-bits
        // - should contain at least 64-bits of entropy: useing ChaCha20 seeded by the OS ensures this
        // - must be unique: uniqueness is likely here and enforced elsewhere by database constraint
        //
        // See: https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
        let mut rng = ChaCha20Rng::from_entropy();
        Secret::new(format!("{:x}", rng.gen::<u128>()))
    }

    pub async fn create(
        db: &Database,
        user_id: Uuid,
    ) -> Result<Secret<String>, tokio_rusqlite::Error> {
        let token = Self::generate_session_token();
        let hashed_token = Secret::new(Sha256::digest(token.expose_secret().as_bytes()).to_vec());

        db.conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("INSERT INTO sessions VALUES (:session_token, :user_id);")?;
                let result = statement.execute(named_params! {
                    ":session_token": hashed_token.expose_secret(),
                    ":user_id": user_id,
                })?;
                Ok(result)
            })
            .await?;

        Ok(token)
    }
}
