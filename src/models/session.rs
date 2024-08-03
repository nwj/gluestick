use crate::db::Database;
use crate::models::prelude::*;
use crate::models::user::User;
use derive_more::From;
use jiff::{Timestamp, ToSpan, Unit};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use secrecy::{ExposeSecret, Secret};
use sha2::{Digest, Sha256};
use tokio_rusqlite::named_params;

pub const SESSION_COOKIE_NAME: &str = "session_token";
const SESSION_ABSOLUTE_TTL_SECONDS: i64 = 1_209_600; // 14 days

#[derive(Debug)]
pub struct Session {
    pub token: HashedSessionToken,
    pub user: User,
    pub created_at: Timestamp,
}

impl Session {
    pub fn new(token: impl Into<HashedSessionToken>, user: User) -> Self {
        Self {
            token: token.into(),
            user,
            created_at: Timestamp::now(),
        }
    }

    pub async fn expire_absolute(db: &Database) -> Result<usize> {
        let expiration_ttl = SESSION_ABSOLUTE_TTL_SECONDS.seconds();
        tracing::trace!(
            "expiring sessions older than {} days",
            expiration_ttl.total(Unit::Day)?
        );
        let expiration_timestamp = Timestamp::now().checked_sub(expiration_ttl)?;
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn
                    .prepare("DELETE FROM sessions WHERE created_at <= :expiration_timestamp;")?;
                let result = statement.execute(
                    named_params! {":expiration_timestamp": expiration_timestamp.as_millisecond()},
                )?;
                Ok(result)
            })
            .await?;

        tracing::trace!(
            "done expiring sessions older than {} days, expired {result} sessions",
            expiration_ttl.total(Unit::Day)?
        );
        Ok(result)
    }

    pub async fn insert(self, db: &Database) -> Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "INSERT INTO sessions VALUES (:session_token, :user_id, :created_at);",
                )?;
                let result = statement.execute(named_params! {
                    ":session_token": self.token,
                    ":user_id": self.user.id,
                    ":created_at": self.created_at.as_millisecond(),
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }
}

#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
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
        Self(Secret::new(format!("{:032x}", rng.gen::<u128>())))
    }
}

impl TryFrom<&str> for SessionToken {
    type Error = std::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        u128::from_str_radix(value, 16)?;
        Ok(Self(Secret::new(value.to_string())))
    }
}

impl ExposeSecret<String> for SessionToken {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[derive(From)]
pub struct HashedSessionToken(Secret<Vec<u8>>);

impl From<&SessionToken> for HashedSessionToken {
    fn from(token: &SessionToken) -> Self {
        let hash = Sha256::digest(token.expose_secret().as_bytes()).to_vec();
        Self(Secret::new(hash))
    }
}

impl std::fmt::Debug for HashedSessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED HashedSessionToken]")
    }
}

impl ExposeSecret<Vec<u8>> for HashedSessionToken {
    fn expose_secret(&self) -> &Vec<u8> {
        self.0.expose_secret()
    }
}

impl ToSql for HashedSessionToken {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.expose_secret().to_sql()
    }
}

impl FromSql for HashedSessionToken {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        Vec::<u8>::column_result(value).map(|vec| Ok(Secret::new(vec).into()))?
    }
}
