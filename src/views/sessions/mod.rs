use crate::controllers::prelude::ErrorTemplate;
use crate::models::session::Session;
use crate::params::prelude::Report;
use crate::params::sessions::CreateSessionParams;
use askama_axum::Template;
use secrecy::ExposeSecret;

#[derive(Clone, Debug, Default, Template)]
#[template(path = "sessions/new.html")]
pub struct NewSessionsTemplate {
    pub session: Option<Session>,
    pub email: String,
    pub password: String,
    pub error_report: Report,
}

impl From<CreateSessionParams> for NewSessionsTemplate {
    fn from(params: CreateSessionParams) -> Self {
        Self {
            session: None,
            email: params.email.into(),
            password: params.password.expose_secret().to_string(),
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
