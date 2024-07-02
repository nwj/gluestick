use crate::db::Database;
use crate::models::invite_code::InviteCode;
use crate::models::user::User;
use crate::params::prelude::*;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct UsernameParam(String);

impl UsernameParam {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for UsernameParam {
    fn from(value: String) -> Self {
        UsernameParam(value)
    }
}

impl From<UsernameParam> for String {
    fn from(value: UsernameParam) -> Self {
        value.0
    }
}

impl Validate for UsernameParam {
    fn validate(&self) -> Result<(), Report> {
        let mut report = Report::new();

        if self.0.chars().count() < 3 {
            report.add("username", "Username must be at least 3 characters long");
        }
        if self.0.chars().count() > 32 {
            report.add("username", "Username may not be longer than 32 characters");
        }
        if !self.0.chars().all(char::is_alphanumeric) {
            report.add(
                "username",
                "Username may only include alphanumeric characters",
            );
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report)
        }
    }
}

impl Verify for UsernameParam {
    type Output = ();

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        let mut report = Report::new();

        if User::find_by_username(db, self.into_inner())
            .await
            .map_err(|e| Error::Other(Box::new(e)))?
            .is_some()
        {
            report.add("username", "Username is already taken");
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct EmailAddressParam(String);

impl EmailAddressParam {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<String> for EmailAddressParam {
    fn from(value: String) -> Self {
        EmailAddressParam(value)
    }
}

impl From<EmailAddressParam> for String {
    fn from(value: EmailAddressParam) -> Self {
        value.0
    }
}

impl Validate for EmailAddressParam {
    fn validate(&self) -> Result<(), Report> {
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
            Err(report)
        }
    }
}

impl Verify for EmailAddressParam {
    type Output = ();

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        let mut report = Report::new();

        if User::find_by_email(db, self.into_inner())
            .await
            .map_err(|e| Error::Other(Box::new(e)))?
            .is_some()
        {
            report.add("email", "Email is already taken");
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report.into())
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct PasswordParam(Secret<String>);

impl PasswordParam {
    pub fn into_inner(self) -> Secret<String> {
        self.0
    }
}

impl From<Secret<String>> for PasswordParam {
    fn from(value: Secret<String>) -> Self {
        PasswordParam(value)
    }
}

impl From<PasswordParam> for Secret<String> {
    fn from(value: PasswordParam) -> Self {
        value.0
    }
}

impl Validate for PasswordParam {
    fn validate(&self) -> Result<(), Report> {
        let mut report = Report::new();

        if self.expose_secret().chars().count() < 8 {
            report.add("password", "Passwords must be at least 8 characters long");
        }
        if self.expose_secret().chars().count() > 256 {
            report.add(
                "password",
                "Passwords may not be longer than 256 characters",
            );
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report)
        }
    }
}

impl ExposeSecret<String> for PasswordParam {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct InviteCodeParam(pub String);

impl InviteCodeParam {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Validate for InviteCodeParam {
    fn validate(&self) -> Result<(), Report> {
        Ok(())
    }
}

impl From<String> for InviteCodeParam {
    fn from(value: String) -> Self {
        InviteCodeParam(value)
    }
}

impl From<InviteCodeParam> for String {
    fn from(value: InviteCodeParam) -> Self {
        value.0
    }
}

impl Verify for InviteCodeParam {
    type Output = InviteCode;

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        let mut report = Report::new();

        if let Some(invite_code) = InviteCode::find(db, self.into_inner())
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
    fn validate(&self) -> Result<(), Report> {
        let mut report = Report::new();

        if let Err(username_report) = self.username.validate() {
            report.merge(username_report);
        }
        if let Err(email_report) = self.email.validate() {
            report.merge(email_report);
        }
        if let Err(password_report) = self.password.validate() {
            report.merge(password_report);
        }
        if let Err(invite_code_report) = self.invite_code.validate() {
            report.merge(invite_code_report);
        }

        if report.is_empty() {
            Ok(())
        } else {
            Err(report)
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
