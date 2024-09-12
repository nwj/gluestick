use crate::controllers::users_controller::ChangePasswordParams;
use crate::controllers::users_controller::CreateUserParams;
use crate::helpers::pagination::CursorPaginationResponse;
use crate::models::api_session::ApiKey;
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::User;

use crate::views::filters;
use askama_axum::Template;
use secrecy::{ExposeSecret, Secret};

#[derive(Clone, Debug, Template)]
#[template(path = "users/new.html")]
pub struct NewUsersTemplate {
    pub session: Option<Session>,
    pub username: String,
    pub username_error_message: Option<String>,
    pub email: String,
    pub email_error_message: Option<String>,
    pub password: Secret<String>,
    pub password_error_message: Option<String>,
    pub invite_code: String,
    pub invite_code_error_message: Option<String>,
}

impl Default for NewUsersTemplate {
    fn default() -> Self {
        Self {
            session: None,
            username: String::default(),
            username_error_message: Option::default(),
            email: String::default(),
            email_error_message: Option::default(),
            password: Secret::new(String::default()),
            password_error_message: Option::default(),
            invite_code: String::default(),
            invite_code_error_message: Option::default(),
        }
    }
}

impl From<CreateUserParams> for NewUsersTemplate {
    fn from(params: CreateUserParams) -> Self {
        Self {
            username: params.username,
            email: params.email,
            password: params.password,
            invite_code: params.invite_code,
            ..Default::default()
        }
    }
}

#[derive(Template)]
#[template(path = "users/show.html")]
pub struct ShowUsersTemplate {
    pub session: Option<Session>,
    pub user: User,
    pub paste_html_pairs: Vec<(Paste, Option<String>)>,
    pub pagination: CursorPaginationResponse,
}

#[derive(Default, Template)]
#[template(path = "users/settings.html")]
pub struct SettingsTemplate {
    pub session: Option<Session>,
    pub api_keys: Vec<ApiKey>,
    pub change_password_form: ChangePasswordFormPartial,
}

#[derive(Clone, Debug, Template)]
#[template(path = "users/partials/change_password_form.html")]
pub struct ChangePasswordFormPartial {
    pub old_password: Secret<String>,
    pub new_password: Secret<String>,
    pub new_password_confirm: Secret<String>,
    pub old_password_error_message: Option<String>,
    pub new_password_error_message: Option<String>,
    pub show_success_message: bool,
}

impl Default for ChangePasswordFormPartial {
    fn default() -> Self {
        Self {
            old_password: Secret::new(String::default()),
            old_password_error_message: Option::default(),
            new_password: Secret::new(String::default()),
            new_password_error_message: Option::default(),
            new_password_confirm: Secret::new(String::default()),
            show_success_message: bool::default(),
        }
    }
}

impl From<ChangePasswordParams> for ChangePasswordFormPartial {
    fn from(params: ChangePasswordParams) -> Self {
        Self {
            old_password: params.old_password,
            new_password: params.new_password,
            new_password_confirm: params.new_password_confirm,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default, Template)]
#[template(path = "users/partials/username_input.html")]
pub struct UsernameInputPartial {
    pub username: String,
    pub username_error_message: Option<String>,
}

impl From<CreateUserParams> for UsernameInputPartial {
    fn from(params: CreateUserParams) -> Self {
        Self {
            username: params.username,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default, Template)]
#[template(path = "users/partials/email_input.html")]
pub struct EmailAddressInputPartial {
    pub email: String,
    pub email_error_message: Option<String>,
}

impl From<CreateUserParams> for EmailAddressInputPartial {
    fn from(params: CreateUserParams) -> Self {
        Self {
            email: params.email,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Template)]
#[template(path = "users/partials/password_input.html")]
pub struct PasswordInputPartial {
    pub password: Secret<String>,
    pub password_error_message: Option<String>,
}

impl Default for PasswordInputPartial {
    fn default() -> Self {
        Self {
            password: Secret::new(String::default()),
            password_error_message: Option::default(),
        }
    }
}

impl From<CreateUserParams> for PasswordInputPartial {
    fn from(params: CreateUserParams) -> Self {
        Self {
            password: params.password,
            ..Default::default()
        }
    }
}
