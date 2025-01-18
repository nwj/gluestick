use crate::controllers::sessions_controller::CreateParams;
use crate::models::session::Session;
use askama_axum::Template;
use secrecy::{ExposeSecret, SecretString};

#[derive(Clone, Debug, Default, Template)]
#[template(path = "sessions/new.html")]
pub struct NewPage {
    pub session: Option<Session>,
    pub email: String,
    pub password: SecretString,
    pub error_message: Option<String>,
}

impl From<CreateParams> for NewPage {
    fn from(params: CreateParams) -> Self {
        Self {
            email: params.email,
            password: params.password,
            ..Default::default()
        }
    }
}
