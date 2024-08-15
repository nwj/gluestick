use crate::db::Database;
use crate::models::invite_code::InviteCode;
use crate::models::user::User;
use crate::params::prelude::*;
use derive_more::{From, Into};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

const MAX_USERNAME_LENGTH: usize = 32;
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 256;

const INVALID_USERNAME_CHARS_MESSAGE: &str = "Username must only contain alphanumeric characters or single hyphens, and may not begin or end with a hyphen";

pub const USERNAME_REPORT_KEY: &str = "username";
pub const EMAIL_REPORT_KEY: &str = "email";
pub const PASSWORD_REPORT_KEY: &str = "password";
pub const INVITE_CODE_REPORT_KEY: &str = "invite_code";

#[derive(Clone, Debug, Deserialize, From, Into, PartialEq)]
#[serde(transparent)]
pub struct UsernameParam(String);

impl UsernameParam {
    pub fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.0.is_empty() {
            report.add(USERNAME_REPORT_KEY, "Username is a required field");
        }
        if self.0.chars().count() > MAX_USERNAME_LENGTH {
            report.add(
                USERNAME_REPORT_KEY,
                format!("Username is too long (maximum is {MAX_USERNAME_LENGTH} characters)"),
            );
        }
        if !self.0.chars().all(|c| c.is_alphanumeric() || c == '-') {
            report.add(USERNAME_REPORT_KEY, INVALID_USERNAME_CHARS_MESSAGE);
        }
        if self.0.starts_with('-') || self.0.ends_with('-') {
            report.add(USERNAME_REPORT_KEY, INVALID_USERNAME_CHARS_MESSAGE);
        }
        if self.0.contains("--") {
            report.add(USERNAME_REPORT_KEY, INVALID_USERNAME_CHARS_MESSAGE);
        }
        if [
            "api",
            "api_sessions",
            "assets",
            "health",
            "login",
            "logout",
            "new",
            "pastes",
            "settings",
            "signup",
        ]
        .into_iter()
        .any(|reserved_name| reserved_name == self.0)
        {
            report.add(USERNAME_REPORT_KEY, "Username is unavailable");
        }

        report.to_result()
    }

    pub async fn check_if_taken(&self, db: &Database) -> Result<()> {
        let username = self.clone();
        match User::find_by_username(db, username).await {
            Ok(None) => Ok(()),
            Ok(Some(_)) => {
                let mut report = Report::new();
                report.add(USERNAME_REPORT_KEY, "Username is already taken");
                Err(report.into())
            }
            Err(e) => Err(Error::Other(Box::new(e))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, From, Into)]
#[serde(transparent)]
pub struct EmailAddressParam(String);

impl EmailAddressParam {
    pub fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.0.is_empty() {
            report.add(EMAIL_REPORT_KEY, "Email is a required field");
        }
        if !self.0.contains('@') {
            report.add(EMAIL_REPORT_KEY, "Email is missing the '@' symbol");
        }
        if self.0.starts_with('@') {
            report.add(
                EMAIL_REPORT_KEY,
                "Email is missing the username part before the '@' symbol",
            );
        }
        if self.0.ends_with('@') {
            report.add(
                EMAIL_REPORT_KEY,
                "Email is missing the domain part after the '@' symbol",
            );
        }

        report.to_result()
    }

    pub async fn check_if_taken(&self, db: &Database) -> Result<()> {
        let email = self.clone();
        match User::find_by_email(db, email).await {
            Ok(None) => Ok(()),
            Ok(Some(_)) => {
                let mut report = Report::new();
                report.add(EMAIL_REPORT_KEY, "Email is already taken");
                Err(report.into())
            }
            Err(e) => Err(Error::Other(Box::new(e))),
        }
    }
}

#[derive(Clone, Debug, Deserialize, From, Into)]
#[serde(transparent)]
pub struct PasswordParam(Secret<String>);

impl PasswordParam {
    pub fn validate(&self) -> Result<()> {
        let mut report = Report::new();

        if self.expose_secret().is_empty() {
            report.add(PASSWORD_REPORT_KEY, "Password is a required field");
        }
        if self.expose_secret().chars().count() < MIN_PASSWORD_LENGTH {
            report.add(
                PASSWORD_REPORT_KEY,
                format!("Password is too short (minimum is {MIN_PASSWORD_LENGTH} characters)"),
            );
        }
        if self.expose_secret().chars().count() > MAX_PASSWORD_LENGTH {
            report.add(
                PASSWORD_REPORT_KEY,
                format!("Password is too long (maximum is {MAX_PASSWORD_LENGTH} characters)"),
            );
        }

        report.to_result()
    }
}

impl ExposeSecret<String> for PasswordParam {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

#[derive(Clone, Deserialize)]
pub struct CreateUserParams {
    pub username: UsernameParam,
    pub email: EmailAddressParam,
    pub password: PasswordParam,
    pub invite_code: String,
}

impl CreateUserParams {
    pub async fn validate_and_check_if_taken(&self, db: &Database) -> Result<()> {
        let mut report = Report::new();

        let mut username_report = Report::new();
        username_report.merge_result(self.username.validate())?;
        if username_report.is_empty() {
            username_report.merge_result(self.username.check_if_taken(db).await)?;
        }

        let mut email_report = Report::new();
        email_report.merge_result(self.email.validate())?;
        if email_report.is_empty() {
            email_report.merge_result(self.email.check_if_taken(db).await)?;
        }

        report.merge(email_report);
        report.merge(username_report);
        report.merge_result(self.password.validate())?;

        report.to_result()
    }

    pub async fn verify_invite_code(&self, db: &Database) -> Result<InviteCode> {
        let mut report = Report::new();
        let invite_code = self.invite_code.clone();

        if let Some(invite_code) = InviteCode::find(db, invite_code)
            .await
            .map_err(|e| Error::Other(Box::new(e)))?
        {
            Ok(invite_code)
        } else {
            report.add(INVITE_CODE_REPORT_KEY, "Invalid invite code");
            Err(report.into())
        }
    }
}
