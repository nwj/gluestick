use crate::controllers::prelude::ErrorTemplate;
use crate::models::session::Session;
use crate::params::prelude::Report;
use crate::params::sessions::CreateSessionParams;
use askama_axum::Template;
use secrecy::{ExposeSecret, Secret};

#[derive(Clone, Debug, Template)]
#[template(path = "sessions/new.html")]
pub struct NewSessionsTemplate {
    pub session: Option<Session>,
    pub email: String,
    pub password: Secret<String>,
    pub error_report: Report,
}

impl Default for NewSessionsTemplate {
    fn default() -> Self {
        Self {
            session: Option::default(),
            email: String::default(),
            password: Secret::new(String::default()),
            error_report: Report::default(),
        }
    }
}

impl From<CreateSessionParams> for NewSessionsTemplate {
    fn from(params: CreateSessionParams) -> Self {
        Self {
            session: None,
            email: params.email,
            password: params.password,
            error_report: Report::default(),
        }
    }
}

impl ErrorTemplate for NewSessionsTemplate {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
    }

    fn with_report(&mut self, report: Report) {
        self.error_report = report;
    }
}
