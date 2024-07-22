use crate::db::Database;
use crate::models::invite_code::InviteCode;
use crate::models::user::User;
use crate::params::prelude::*;
use derive_more::{From, Into};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

const USERNAME_CHAR_VALIDATION_FAILURE_MESSAGE: &str = "Username may only contain alphanumeric characters or single hyphens, and cannot begin or end with a hyphen";

#[derive(Clone, Debug, Deserialize, From, Into, PartialEq)]
#[serde(transparent)]
pub struct UsernameParam(String);

impl Validate for UsernameParam {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.0.chars().count() < 1 {
            report.add("username", "Username is too short (minimum is 1 character)");
        }
        if self.0.chars().count() > 32 {
            report.add(
                "username",
                "Username is too long (maximum is 32 characters)",
            );
        }
        if !self.0.chars().all(|c| c.is_alphanumeric() || c == '-') {
            report.add("username", USERNAME_CHAR_VALIDATION_FAILURE_MESSAGE);
        }
        if self.0.starts_with('-') || self.0.ends_with('-') {
            report.add("username", USERNAME_CHAR_VALIDATION_FAILURE_MESSAGE);
        }
        if self.0.contains("--") {
            report.add("username", USERNAME_CHAR_VALIDATION_FAILURE_MESSAGE);
        }
        if [
            "api",
            "api_sessions",
            "assets",
            "health",
            "login",
            "logout",
            "pastes",
            "settings",
            "signup",
        ]
        .into_iter()
        .any(|reserved_name| reserved_name == self.0)
        {
            report.add("username", "Username is unavailable");
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
    }
}

impl Verify for UsernameParam {
    type Output = ();

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        match User::find_by_username(db, self).await {
            Ok(None) => Ok(()),
            Ok(Some(_)) => {
                let mut report = Report::new();
                report.add("username", "Username is already taken");
                Err(report.into())
            }
            Err(e) => Err(Error::Other(Box::new(e))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, From, Into)]
#[serde(transparent)]
pub struct EmailAddressParam(String);

impl Validate for EmailAddressParam {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if !self.0.contains('@') {
            report.add("email", "Email is missing the '@' symbol");
        }
        if self.0.starts_with('@') {
            report.add(
                "email",
                "Email is missing the username part before the '@' symbol",
            );
        }
        if self.0.ends_with('@') {
            report.add(
                "email",
                "Email is missing the domain part after the '@' symbol",
            );
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
    }
}

impl Verify for EmailAddressParam {
    type Output = ();

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        match User::find_by_email(db, self.into()).await {
            Ok(None) => Ok(()),
            Ok(Some(_)) => {
                let mut report = Report::new();
                report.add("email", "Email is already taken");
                Err(report.into())
            }
            Err(e) => Err(Error::Other(Box::new(e))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, From, Into)]
#[serde(transparent)]
pub struct PasswordParam(Secret<String>);

impl Validate for PasswordParam {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.expose_secret().chars().count() < 8 {
            report.add(
                "password",
                "Password is too short (minimum is 8 characters)",
            );
        }
        if self.expose_secret().chars().count() > 256 {
            report.add(
                "password",
                "Password is too long (maximum is 256 characters)",
            );
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

#[derive(Clone, Debug, Deserialize, From, Into)]
#[serde(transparent)]
pub struct InviteCodeParam(pub String);

impl Validate for InviteCodeParam {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl Verify for InviteCodeParam {
    type Output = InviteCode;

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        let mut report = Report::new();

        if let Some(invite_code) = InviteCode::find(db, self)
            .await
            .map_err(|e| Error::Other(Box::new(e)))?
        {
            Ok(invite_code)
        } else {
            report.add("invite_code", "Invalid invite code");
            Err(report.into())
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct CreateUserParams {
    pub username: UsernameParam,
    pub email: EmailAddressParam,
    pub password: PasswordParam,
    pub invite_code: InviteCodeParam,
}

impl Validate for CreateUserParams {
    fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        match self.username.validate() {
            Err(Error::Report(username_report)) => report.merge(username_report),
            Err(Error::Other(e)) => return Err(Error::Other(e)),
            _ => {}
        };
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
        match self.invite_code.validate() {
            Err(Error::Report(invite_code_report)) => report.merge(invite_code_report),
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

impl Verify for CreateUserParams {
    type Output = InviteCode;

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        let mut report = Report::new();

        match self.username.verify(db).await {
            Err(Error::Report(username_report)) => report.merge(username_report),
            Err(Error::Other(e)) => return Err(Error::Other(e)),
            _ => {}
        };
        match self.email.verify(db).await {
            Err(Error::Report(email_report)) => report.merge(email_report),
            Err(Error::Other(e)) => return Err(Error::Other(e)),
            _ => {}
        };
        match self.invite_code.verify(db).await {
            Err(Error::Report(invite_code_report)) => {
                report.merge(invite_code_report);
                Err(report.into())
            }
            Err(Error::Other(e)) => Err(Error::Other(e)),
            Ok(invite_code) => {
                if report.is_empty() {
                    Ok(invite_code)
                } else {
                    Err(report.into())
                }
            }
        }
    }
}
