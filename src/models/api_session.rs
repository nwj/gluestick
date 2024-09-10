use crate::db::Database;
use crate::models::prelude::*;
use crate::models::user::User;
use derive_more::{Display, From};
use jiff::Timestamp;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, Type, ValueRef};
use rusqlite::{Row, Transaction, TransactionBehavior};
use secrecy::{ExposeSecret, Secret};
use sha2::{Digest, Sha256};
use tokio_rusqlite::named_params;
use uuid::Uuid;

pub const API_KEY_HEADER_NAME: &str = "X-GLUESTICK-API-KEY";

#[derive(Debug, Display)]
#[display("{{ api_key: {api_key}, user: {user} }}")]
pub struct ApiSession {
    pub api_key: ApiKey,
    pub user: User,
}

impl ApiSession {
    pub async fn find_by_unhashed_key(
        db: &Database,
        unhashed_key: &UnhashedKey,
    ) -> Result<Option<Self>> {
        let hashed_key = HashedKey::from(unhashed_key);
        let maybe_api_session = db
            .conn
            .call(move |conn| {
                let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
                let maybe_api_session = Self::tx_find_by_hashed_key(&tx, &hashed_key)?;
                if let Some(ref api_session) = maybe_api_session {
                    api_session.api_key.tx_touch(&tx)?;
                }
                tx.commit()?;
                Ok(maybe_api_session)
            })
            .await?;

        Ok(maybe_api_session)
    }

    pub fn tx_find_by_hashed_key(
        tx: &Transaction,
        key: &HashedKey,
    ) -> tokio_rusqlite::Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r"SELECT
                users.id, users.username, users.email, users.password, users.created_at, users.updated_at,
                api_keys.id, api_keys.name, api_keys.key, api_keys.user_id, api_keys.created_at, api_keys.last_used_at
            FROM users JOIN api_keys ON users.id = api_keys.user_id
            WHERE api_keys.key = :key;"
        )?;
        let mut rows = stmt.query(named_params! {":key": key})?;
        match rows.next()? {
            Some(row) => {
                let user = User::from_sql_row(row)?;
                let api_key = ApiKey::from_sql_row(row, 6)?;
                Ok(Some(ApiSession { api_key, user }))
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug, Display)]
#[display("{{ id: {id}, user_id: {user_id} }}")]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key: HashedKey,
    pub user_id: Uuid,
    pub created_at: Timestamp,
    pub last_used_at: Timestamp,
}

impl ApiKey {
    pub fn new(user_id: Uuid) -> (UnhashedKey, Self) {
        let now = Timestamp::now();
        let unhashed_key = UnhashedKey::generate();
        let last_four =
            &unhashed_key.expose_secret()[unhashed_key.expose_secret().len().saturating_sub(4)..];
        let api_key = Self {
            id: Uuid::now_v7(),
            name: format!("API Key ending in '{last_four}'"),
            key: HashedKey::from(&unhashed_key),
            user_id,
            created_at: now,
            last_used_at: now,
        };
        (unhashed_key, api_key)
    }

    pub fn from_sql_row(row: &Row, offset: usize) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(offset)?,
            name: row.get(1 + offset)?,
            key: row.get(offset + 2)?,
            user_id: row.get(3 + offset)?,
            created_at: Timestamp::from_millisecond(row.get(4 + offset)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(4 + offset, Type::Integer, Box::new(e))
            })?,
            last_used_at: Timestamp::from_millisecond(row.get(5 + offset)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(5 + offset, Type::Integer, Box::new(e))
            })?,
        })
    }

    pub async fn all_for_user_id(db: &Database, user_id: Uuid) -> Result<Vec<Self>> {
        let api_keys: Vec<_> = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    r"SELECT id, name, key, user_id, created_at, last_used_at FROM api_keys
                    WHERE user_id = :user_id ORDER BY id DESC;",
                )?;
                let api_key_iter = statement
                    .query_map(named_params! {":user_id": user_id}, |k| {
                        Self::from_sql_row(k, 0)
                    })?;
                Ok(api_key_iter.collect::<Result<Vec<_>, _>>()?)
            })
            .await?;
        Ok(api_keys)
    }

    pub async fn find_scoped_by_user_id(
        db: &Database,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<ApiKey>> {
        let maybe_api_key = db
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    r"SELECT id, name, key, user_id, created_at, last_used_at FROM api_keys
                    WHERE id = :id AND user_id = :user_id;",
                )?;
                let mut rows = stmt.query(named_params! {":id": id, ":user_id": user_id})?;
                match rows.next()? {
                    Some(row) => Ok(Some(ApiKey::from_sql_row(row, 0)?)),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(maybe_api_key)
    }

    pub async fn insert(self, db: &Database) -> Result<usize> {
        tracing::info!("inserting api key {self}");
        let result = db
            .conn
            .call(move |conn| {
                let mut statement = conn.prepare(
                    "INSERT INTO api_keys VALUES (:id, :name, :key, :user_id, :created_at, :last_used_at);",
                )?;
                let result = statement.execute(named_params! {
                    ":id": self.id,
                    ":name": self.name,
                    ":key": self.key,
                    ":user_id": self.user_id,
                    ":created_at": self.created_at.as_millisecond(),
                    ":last_used_at": self.last_used_at.as_millisecond(),
                })?;
                Ok(result)
            })
            .await?;

        Ok(result)
    }

    pub async fn delete(self, db: &Database) -> Result<usize> {
        tracing::info!("deleting api key {self}");
        let result = db
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare("DELETE FROM api_keys WHERE id = :id;")?;
                let result = stmt.execute(named_params! {":id": self.id})?;
                Ok(result)
            })
            .await?;
        Ok(result)
    }

    pub fn tx_touch(&self, tx: &Transaction) -> tokio_rusqlite::Result<()> {
        let mut stmt =
            tx.prepare("UPDATE api_keys SET last_used_at = :last_used_at WHERE key = :key;")?;
        stmt.execute(
            named_params! {":last_used_at": Timestamp::now().as_millisecond(), ":key": self.key},
        )?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct UnhashedKey(Secret<String>);

impl UnhashedKey {
    pub fn generate() -> Self {
        let mut rng = ChaCha20Rng::from_entropy();
        Self(Secret::new(format!("{:032x}", rng.gen::<u128>())))
    }
}

impl TryFrom<&str> for UnhashedKey {
    type Error = std::num::ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        u128::from_str_radix(value, 16)?;
        Ok(Self(Secret::new(value.to_string())))
    }
}

impl ExposeSecret<String> for UnhashedKey {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[derive(From)]
pub struct HashedKey(Secret<Vec<u8>>);

impl From<&UnhashedKey> for HashedKey {
    fn from(key: &UnhashedKey) -> Self {
        let hash = Sha256::digest(key.expose_secret().as_bytes()).to_vec();
        Self(Secret::new(hash))
    }
}

impl ExposeSecret<Vec<u8>> for HashedKey {
    fn expose_secret(&self) -> &Vec<u8> {
        self.0.expose_secret()
    }
}

impl std::fmt::Debug for HashedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED HashedKey]")
    }
}

impl ToSql for HashedKey {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.expose_secret().to_sql()
    }
}

impl FromSql for HashedKey {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        Vec::<u8>::column_result(value).map(|vec| Ok(Secret::new(vec).into()))?
    }
}
