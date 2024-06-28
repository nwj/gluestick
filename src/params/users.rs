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

impl Validate for UsernameParam {
    fn validate(&self) -> Result<()> {
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

#[derive(Clone, Debug, Deserialize)]
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
            Err(report)
        }
    }
}

impl EmailAddressParam {
    pub fn into_inner(self) -> String {
        self.0
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

impl Validate for PasswordParam {
    fn validate(&self) -> Result<()> {
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
    fn validate(&self) -> Result<()> {
        Ok(())
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
