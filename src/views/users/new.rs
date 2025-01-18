use crate::controllers::users_controller::CreateParams;
use crate::models::session::Session;
use askama_axum::Template;
use secrecy::{ExposeSecret, SecretString};

#[derive(Clone, Debug, Default, Template)]
#[template(path = "users/new.html")]
pub struct NewPage {
    pub session: Option<Session>,
    pub username: String,
    pub username_error_message: Option<String>,
    pub email: String,
    pub email_error_message: Option<String>,
    pub password: SecretString,
    pub password_error_message: Option<String>,
    pub invite_code: String,
    pub invite_code_error_message: Option<String>,
}

impl From<CreateParams> for NewPage {
    fn from(params: CreateParams) -> Self {
        Self {
            username: params.username,
            email: params.email,
            password: params.password,
            invite_code: params.invite_code,
            ..Default::default()
        }
    }
}

// TODO: replace this partial with a block fragment once askama 0.13.0 releases
#[derive(Clone, Debug, Default, Template)]
#[template(path = "users/partials/username_input.html")]
pub struct UsernameInputPartial {
    pub username: String,
    pub username_error_message: Option<String>,
}

impl From<CreateParams> for UsernameInputPartial {
    fn from(params: CreateParams) -> Self {
        Self {
            username: params.username,
            ..Default::default()
        }
    }
}

// TODO: replace this partial with a block fragment once askama 0.13.0 releases
#[derive(Clone, Debug, Default, Template)]
#[template(path = "users/partials/email_input.html")]
pub struct EmailInputPartial {
    pub email: String,
    pub email_error_message: Option<String>,
}

impl From<CreateParams> for EmailInputPartial {
    fn from(params: CreateParams) -> Self {
        Self {
            email: params.email,
            ..Default::default()
        }
    }
}

// TODO: replace this partial with a block fragment once askama 0.13.0 releases
#[derive(Clone, Debug, Default, Template)]
#[template(path = "users/partials/password_input.html")]
pub struct PasswordInputPartial {
    pub password: SecretString,
    pub password_error_message: Option<String>,
}

impl From<CreateParams> for PasswordInputPartial {
    fn from(params: CreateParams) -> Self {
        Self {
            password: params.password,
            ..Default::default()
        }
    }
}
