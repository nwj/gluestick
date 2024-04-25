use std::env;
use std::path::{Path, PathBuf};

const DATABASE_PATH_ENV_VAR: &str = "GLUESTICK_DB_PATH";
const PORT_ENV_VAR: &str = "GLUESTICK_PORT";

const DATABASE_PATH_DEFAULT: &str = "gluestick.db";
const PORT_DEFAULT: u16 = 3000;

#[derive(Debug)]
pub struct Config {
    database_path: PathBuf,
    port: u16,
}

impl Config {
    pub fn parse() -> Result<Config, Error> {
        let database_path = Self::parse_database_path()?;
        let port = Self::parse_port()?;

        Ok(Config {
            database_path,
            port,
        })
    }

    fn parse_database_path() -> Result<PathBuf, DatabasePathError> {
        match env::var(DATABASE_PATH_ENV_VAR) {
            Ok(s) => Ok(PathBuf::from(s)),
            Err(env::VarError::NotPresent) => Ok(PathBuf::from(DATABASE_PATH_DEFAULT)),
            Err(err @ env::VarError::NotUnicode(_)) => Err(err.into()),
        }
    }

    fn parse_port() -> Result<u16, PortError> {
        match env::var(PORT_ENV_VAR) {
            Ok(s) => s.parse().map_err(Into::into),
            Err(env::VarError::NotPresent) => Ok(PORT_DEFAULT),
            Err(err @ env::VarError::NotUnicode(_)) => Err(err.into()),
        }
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    DatabasePathError(#[from] DatabasePathError),
    PortError(#[from] PortError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum DatabasePathError {
    InvalidUnicode(#[from] env::VarError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum PortError {
    InvalidValue(#[from] std::num::ParseIntError),
    InvalidUnicode(#[from] env::VarError),
}
