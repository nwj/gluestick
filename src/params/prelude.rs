use crate::db::Database;
use std::collections::HashMap;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Report(#[from] Report),

    #[error("other: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Clone, Debug, Default, thiserror::Error)]
pub struct Report {
    errors: HashMap<String, Vec<String>>,
}

impl Report {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add(&mut self, field: impl Into<String>, error: impl Into<String>) {
        let field = field.into();
        let error = error.into();
        self.errors.entry(field).or_default().push(error);
    }

    pub fn get(&self, field: impl Into<String>) -> Option<&Vec<String>> {
        let field = field.into();
        self.errors.get(&field)
    }

    pub fn get_first(&self, field: impl Into<String>) -> Option<&String> {
        self.get(field).and_then(|vec| vec.first())
    }

    pub fn merge(&mut self, other: Self) {
        for (field, errors) in other.errors {
            self.errors.entry(field).or_default().extend(errors);
        }
    }

    pub fn to_result(self) -> Result<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self.into())
        }
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (field, errors) in &self.errors {
            writeln!(f, "{field}:")?;
            for error in errors {
                writeln!(f, " - {error}")?;
            }
        }
        Ok(())
    }
}

pub trait Validate {
    fn validate(&self) -> Result<()>;
}

#[allow(async_fn_in_trait)]
pub trait Verify {
    type Output;
    async fn verify(self, db: &Database) -> Result<Self::Output>;
}
