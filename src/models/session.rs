use crate::db::Database;
use crate::models::prelude::*;
use crate::models::user::User;
use derive_more::From;
use jiff::{Timestamp, ToSpan, Unit};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, Type, ValueRef};
use rusqlite::{Row, Transaction, TransactionBehavior};
use secrecy::{ExposeSecret, Secret};
use sha2::{Digest, Sha256};
use tokio_rusqlite::named_params;
use uuid::Uuid;

pub const SESSION_COOKIE_NAME: &str = "session_token";
const ABSOLUTE_SESSION_TTL_SECONDS: i64 = 1_209_600; // 14 days
const IDLE_SESSION_TTL_SECONDS: i64 = 28_800; // 8 hours

#[derive(Debug)]
pub struct Session {
    pub session_token: SessionToken,
    pub user: User,
}

impl Session {
    pub async fn find_by_unhashed_token(
        db: &Database,
        unhashed_token: &UnhashedToken,
    ) -> Result<Option<Self>> {
        let hashed_token = HashedToken::from(unhashed_token);
        let maybe_session = db
            .conn
            .call(move |conn| {
                let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
                let maybe_session = Self::tx_find_by_hashed_token(&tx, &hashed_token)?;
                if let Some(ref session) = maybe_session {
                    session.session_token.tx_touch(&tx)?;
                }
                tx.commit()?;
                Ok(maybe_session)
            })
            .await?;

        Ok(maybe_session)
    }

    pub fn tx_find_by_hashed_token(
        tx: &Transaction,
        token: &HashedToken,
    ) -> tokio_rusqlite::Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r"SELECT
                users.id, users.username, users.email, users.password, users.created_at, users.updated_at,
                session_tokens.token, session_tokens.user_id, session_tokens.created_at, session_tokens.last_used_at
            FROM users JOIN session_tokens ON users.id = session_tokens.user_id
            WHERE session_tokens.token = :token;",
        )?;
        let mut rows = stmt.query(named_params! {":token": token})?;
        match rows.next()? {
            Some(row) => {
                let user = User::from_sql_row(row)?;
                let session_token = SessionToken::from_sql_row(row, 6)?;
                Ok(Some(Self {
                    session_token,
                    user,
                }))
            }
            None => Ok(None),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct SessionToken {
    pub token: HashedToken,
    pub user_id: Uuid,
    pub created_at: Timestamp,
    pub last_used_at: Timestamp,
}

impl SessionToken {
    pub fn new(user_id: Uuid) -> (UnhashedToken, Self) {
        let unhashed_token = UnhashedToken::generate();
        let session_token = Self {
            token: HashedToken::from(&unhashed_token),
            user_id,
            created_at: Timestamp::now(),
            last_used_at: Timestamp::now(),
        };
        (unhashed_token, session_token)
    }

    pub fn from_sql_row(row: &Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Self {
            token: row.get(offset)?,
            user_id: row.get(1 + offset)?,
            created_at: Timestamp::from_millisecond(row.get(2 + offset)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(2 + offset, Type::Integer, Box::new(e))
            })?,
            last_used_at: Timestamp::from_millisecond(row.get(3 + offset)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(3 + offset, Type::Integer, Box::new(e))
            })?,
        })
    }

    pub async fn expire_absolute(db: &Database) -> Result<usize> {
        let expiration_ttl = ABSOLUTE_SESSION_TTL_SECONDS.seconds();
        tracing::trace!(
            "expiring sessions older than {} days",
            expiration_ttl.total(Unit::Day)?
        );
        let expiration_timestamp = Timestamp::now().checked_sub(expiration_ttl)?;
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "DELETE FROM session_tokens WHERE created_at < :expiration_timestamp;",
                )?;
                let result = statement.execute(
                    named_params! {":expiration_timestamp": expiration_timestamp.as_millisecond()},
                )?;
                Ok(result)
            })
            .await?;

        tracing::trace!("done expiring old sessions, expired {result} sessions");
        Ok(result)
    }

    pub async fn expire_idle(db: &Database) -> Result<usize> {
        let expiration_ttl = IDLE_SESSION_TTL_SECONDS.seconds();
        tracing::trace!(
            "expiring sessions idle for more than {} hours",
            expiration_ttl.total(Unit::Hour)?
        );
        let expiration_timestamp = Timestamp::now().checked_sub(expiration_ttl)?;
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "DELETE FROM session_tokens WHERE last_used_at < :expiration_timestamp;",
                )?;
                let result = statement.execute(
                    named_params! {":expiration_timestamp": expiration_timestamp.as_millisecond()},
                )?;
                Ok(result)
            })
            .await?;

        tracing::trace!("done expiring idle sessions, expired {result} sessions");
        Ok(result)
    }

    pub async fn insert(self, db: &Database) -> Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "INSERT INTO session_tokens VALUES (:token, :user_id, :created_at, :last_used_at);",
                )?;
                let result = statement.execute(named_params! {
                    ":token": self.token,
                    ":user_id": self.user_id,
                    ":created_at": self.created_at.as_millisecond(),
                    ":last_used_at": self.last_used_at.as_millisecond(),
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub fn tx_touch(&self, tx: &Transaction) -> tokio_rusqlite::Result<()> {
        let mut stmt = tx.prepare(
            "UPDATE session_tokens SET last_used_at = :last_used_at WHERE token = :token;",
        )?;
        stmt.execute(
            named_params! {":last_used_at": Timestamp::now().as_millisecond(), ":token": self.token},
        )?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct UnhashedToken(Secret<String>);

impl UnhashedToken {
    pub fn generate() -> Self {
        // The OWASP checklist for session tokens:
        // - has a size of at least 128-bits: ours is 128-bits
        // - contains at least 64-bits of entropy: use of ChaCha20 seeded by the OS should ensure this
        // - must be unique: uniqueness is statistically likely here, but enforced elsewhere by database constraint
        //
        // See: https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
        let mut rng = ChaCha20Rng::from_entropy();
        Self(Secret::new(format!("{:032x}", rng.gen::<u128>())))
    }
}

impl TryFrom<&str> for UnhashedToken {
    type Error = std::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        u128::from_str_radix(value, 16)?;
        Ok(Self(Secret::new(value.to_string())))
    }
}

impl ExposeSecret<String> for UnhashedToken {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[derive(From)]
pub struct HashedToken(Secret<Vec<u8>>);

impl From<&UnhashedToken> for HashedToken {
    fn from(token: &UnhashedToken) -> Self {
        let hash = Sha256::digest(token.expose_secret().as_bytes()).to_vec();
        Self(Secret::new(hash))
    }
}

impl std::fmt::Debug for HashedToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED HashedToken]")
    }
}

impl ExposeSecret<Vec<u8>> for HashedToken {
    fn expose_secret(&self) -> &Vec<u8> {
        self.0.expose_secret()
    }
}

impl ToSql for HashedToken {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.expose_secret().to_sql()
    }
}

impl FromSql for HashedToken {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        Vec::<u8>::column_result(value).map(|vec| Ok(Secret::new(vec).into()))?
    }
}
