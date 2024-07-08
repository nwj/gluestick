use crate::db::Database;
use crate::models::user::User;
use crate::params::prelude::*;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

// Throughout this module, we use this message, rather than more specific error feedback for
// validation and verification errors. The reason for this is that we don't want to provide
// specific feedback that would leak information to an attacker that is attempting to brute-force
// authentication.
const AUTH_FAILURE_MESSAGE: &str = "Incorrect email or password";

#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct EmailAddressParam(String);

impl EmailAddressParam {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for EmailAddressParam {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<EmailAddressParam> for String {
    fn from(value: EmailAddressParam) -> Self {
        value.0
    }
}

impl Validate for EmailAddressParam {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if !self.0.contains('@') {
            report.add("self", AUTH_FAILURE_MESSAGE);
        }
        if self.0.starts_with('@') {
            report.add("self", AUTH_FAILURE_MESSAGE);
        }
        if self.0.ends_with('@') {
            report.add("self", AUTH_FAILURE_MESSAGE);
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
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
        let mut report = Report::new();

        if self.expose_secret().chars().count() < 8 {
            report.add("self", AUTH_FAILURE_MESSAGE);
        }
        if self.expose_secret().chars().count() > 256 {
            report.add("self", AUTH_FAILURE_MESSAGE);
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
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

impl Verify for CreateSessionParams {
    type Output = User;

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        let mut report = Report::new();
        report.add("self", AUTH_FAILURE_MESSAGE);

        let user = User::find_by_email(db, self.email.into_inner())
            .await
            .map_err(|e| Error::Other(Box::new(e)))?;

        if let Some(user) = user {
            user.verify_password(self.password.expose_secret())
                .map_err(|_| Error::Report(report))?;

            Ok(user)
        } else {
            Err(report.into())
        }
    }
}
