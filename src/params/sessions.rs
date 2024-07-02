use crate::params::prelude::*;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct EmailAddressParam(String);

impl EmailAddressParam {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Validate for EmailAddressParam {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct PasswordParam(Secret<String>);

impl PasswordParam {
    pub fn into_inner(self) -> Secret<String> {
        self.0
    }
}

impl Validate for PasswordParam {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl ExposeSecret<String> for PasswordParam {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[derive(Clone, Deserialize)]
pub struct CreateSessionParams {
    pub email: EmailAddressParam,
    pub password: PasswordParam,
}

impl Validate for CreateSessionParams {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        match self.email.validate() {
            Err(Error::Report(email_report)) => report.merge(email_report),
            Err(Error::Other(e)) => return Err(Error::Other(e)),
            _ => {}
        };
        match self.password.validate() {
            Err(Error::Report(password_report)) => report.merge(password_report),
            Err(Error::Other(e)) => return Err(Error::Other(e)),
            _ => {}
        };

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
    }
}
