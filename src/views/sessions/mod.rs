use crate::controllers::sessions_controller::CreateSessionParams;
use crate::models::session::Session;
use askama_axum::Template;
use secrecy::{ExposeSecret, Secret};

#[derive(Clone, Debug, Template)]
#[template(path = "sessions/new.html")]
pub struct NewSessionsTemplate {
    pub session: Option<Session>,
    pub email: String,
    pub password: Secret<String>,
    pub error_message: Option<String>,
}

impl Default for NewSessionsTemplate {
    fn default() -> Self {
        Self {
            session: None,
            email: String::default(),
            password: Secret::new(String::default()),
            error_message: Option::default(),
        }
    }
}

impl From<CreateSessionParams> for NewSessionsTemplate {
    fn from(params: CreateSessionParams) -> Self {
        Self {
            email: params.email,
            password: params.password,
            ..Default::default()
        }
    }
}
