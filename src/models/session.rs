use crate::{auth::SessionToken, db::Database};
use secrecy::ExposeSecret;
use tokio_rusqlite::named_params;
use uuid::Uuid;

#[allow(dead_code)]
pub struct Session {
    session_token: SessionToken,
    user_id: Uuid,
}

impl Session {
    pub async fn insert(
        db: &Database,
        token: SessionToken,
        user_id: Uuid,
    ) -> Result<usize, tokio_rusqlite::Error> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement =
                    conn.prepare("INSERT INTO sessions VALUES (:session_token, :user_id);")?;
                let result = statement.execute(named_params! {
                    ":session_token": token.to_hash().expose_secret(),
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
