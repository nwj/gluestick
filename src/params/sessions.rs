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
    fn validate(&self) -> Result<(), Report> {
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
    fn validate(&self) -> Result<(), Report> {
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
    fn validate(&self) -> Result<(), Report> {
        let mut report = Report::new();

        if let Err(email_report) = self.email.validate() {
            report.merge(email_report);
        }
        if let Err(password_report) = self.password.validate() {
            report.merge(password_report);
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report)
        }
    }
}
