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

    pub fn merge_result(&mut self, result: Result<()>) -> Result<()> {
        match result {
            Err(Error::Report(report)) => {
                self.merge(report);
                Ok(())
            }
            Err(Error::Other(e)) => Err(Error::Other(e)),
            _ => Ok(()),
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
