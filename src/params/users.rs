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

impl Verify for UsernameParam {
    type Output = ();

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        let mut report = Report::new();

        if User::find_by_username(db, self.into_inner())
            .await
            .unwrap()
            .is_some()
        {
            report.add("username", "Username is already taken");
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

impl EmailAddressParam {
    pub fn into_inner(self) -> String {
        self.0
    }
}

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

impl Verify for EmailAddressParam {
    type Output = ();

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        let mut report = Report::new();

        if User::find_by_email(db, self.into_inner())
            .await
            .unwrap()
            .is_some()
        {
            report.add("email", "Email is already taken");
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

impl Verify for InviteCodeParam {
    type Output = InviteCode;

    async fn verify(self, db: &Database) -> Result<Self::Output> {
        let mut report = Report::new();

        if let Some(invite_code) = InviteCode::find(db, self.into_inner()).await.unwrap() {
            Ok(invite_code)
        } else {
            report.add("invite_code", "Invalid invite code");
            Err(report)
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

        if let Err(username_report) = self.username.verify(db).await {
            report.merge(username_report);
        }
        if let Err(email_report) = self.email.verify(db).await {
            report.merge(email_report);
        }

        match self.invite_code.verify(db).await {
            Err(invite_code_report) => {
                report.merge(invite_code_report);
                Err(report)
            }
            Ok(code) => {
                if report.is_empty() {
                    Ok(code)
                } else {
                    Err(report)
                }
            }
        }
    }
}
