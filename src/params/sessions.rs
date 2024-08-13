use crate::db::Database;
use crate::models::user::User;
use crate::params::prelude::*;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

const AUTH_FAILURE_MESSAGE: &str = "Incorrect email or password";
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 256;

pub const SELF_REPORT_KEY: &str = "self";

#[derive(Clone, Deserialize)]
pub struct CreateSessionParams {
    pub email: String,
    pub password: Secret<String>,
}

impl CreateSessionParams {
    pub async fn authenticate(self, db: &Database) -> Result<User> {
        if !self.email.contains('@') {
            return Err(Self::report().into());
        }
        if self.email.starts_with('@') {
            return Err(Self::report().into());
        }
        if self.email.ends_with('@') {
            return Err(Self::report().into());
        }

        if self.password.expose_secret().chars().count() < MIN_PASSWORD_LENGTH {
            return Err(Self::report().into());
        }
        if self.password.expose_secret().chars().count() > MAX_PASSWORD_LENGTH {
            return Err(Self::report().into());
        }

        let user = User::find_by_email(db, self.email)
            .await
            .map_err(|e| Error::Other(Box::new(e)))?;
        match user {
            Some(user) if user.verify_password(self.password.expose_secret()).is_ok() => Ok(user),
            _ => Err(Self::report().into()),
        }
    }

    fn report() -> Report {
        let mut report = Report::new();
        report.add(SELF_REPORT_KEY, AUTH_FAILURE_MESSAGE);
        report
    }
}
