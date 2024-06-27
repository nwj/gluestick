use crate::db::Database;
use crate::models::prelude::*;
use rusqlite::named_params;
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

#[derive(Debug, Default)]
pub struct InviteCode(String);

impl InviteCode {
    pub async fn find(db: &Database, code: String) -> Result<Option<Self>> {
        let optional_code = db
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare("SELECT code FROM invite_codes WHERE code = :code;")?;
                let mut rows = stmt.query(named_params! {":code": code})?;
                match rows.next()? {
                    Some(row) => Ok(Some(row.get(0)?)),
                    None => Ok(None),
                }
            })
            .await?;

        Ok(optional_code)
    }

    pub async fn delete(self, db: &Database) -> Result<usize> {
        let result = db
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare("DELETE FROM invite_codes WHERE code = :code;")?;
                let result = stmt.execute(named_params! {":code": self})?;
                Ok(result)
            })
            .await?;
        Ok(result)
    }
}

impl ToSql for InviteCode {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        self.0.to_sql()
    }
}

impl FromSql for InviteCode {
    fn column_result(value: ValueRef) -> FromSqlResult<Self> {
        String::column_result(value).map(|string| Ok(Self(string)))?
    }
}
