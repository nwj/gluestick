use std::env;

const PORT_ENV_VAR: &str = "GLUESTICK_PORT";

const PORT_DEFAULT: u16 = 3000;

#[derive(Debug)]
pub struct Config {
    port: u16,
}

impl Config {
    pub fn parse() -> Result<Config, Error> {
        let port = Self::parse_port()?;

        Ok(Config { port })
    }

    fn parse_port() -> Result<u16, PortParseError> {
        match env::var(PORT_ENV_VAR) {
            Ok(s) => s.parse().map_err(Into::into),
            Err(env::VarError::NotPresent) => Ok(PORT_DEFAULT),
            Err(err @ env::VarError::NotUnicode(_)) => Err(err.into()),
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    PortError(#[from] PortParseError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum PortParseError {
    InvalidValue(#[from] std::num::ParseIntError),
    InvalidUnicode(#[from] env::VarError),
}
