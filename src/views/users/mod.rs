use crate::controllers::prelude::{ErrorTemplate, ErrorTemplate2};
use crate::controllers::users::ChangePasswordParams;
use crate::helpers::pagination::CursorPaginationResponse;
use crate::models::api_session::ApiKey;
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::User;
use crate::params::prelude::Report;
use crate::params::users::{
    CreateUserParams, EMAIL_REPORT_KEY, INVITE_CODE_REPORT_KEY, PASSWORD_REPORT_KEY,
    USERNAME_REPORT_KEY,
};
use crate::views::filters;
use askama_axum::Template;
use secrecy::{ExposeSecret, Secret};

#[derive(Clone, Debug, Template)]
#[template(path = "users/new.html")]
pub struct NewUsersTemplate {
    pub session: Option<Session>,
    pub username: String,
    pub email: String,
    pub password: Secret<String>,
    pub invite_code: String,
    pub error_report: Report,
}

impl Default for NewUsersTemplate {
    fn default() -> Self {
        Self {
            session: Option::default(),
            username: String::default(),
            email: String::default(),
            password: Secret::new(String::default()),
            invite_code: String::default(),
            error_report: Report::default(),
        }
    }
}

impl From<CreateUserParams> for NewUsersTemplate {
    fn from(params: CreateUserParams) -> Self {
        Self {
            session: None,
            username: params.username.into(),
            email: params.email.into(),
            password: params.password.into(),
            invite_code: params.invite_code,
            error_report: Report::default(),
        }
    }
}

impl ErrorTemplate for NewUsersTemplate {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
    }

    fn with_report(&mut self, report: Report) {
        self.error_report = report;
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

impl ErrorTemplate2 for ChangePasswordFormPartial {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
    }
}

#[derive(Clone, Debug, Default, Template)]
#[template(path = "users/partials/username_input.html")]
pub struct UsernameInputPartial {
    pub username: String,
    pub error_report: Report,
}

impl From<CreateUserParams> for UsernameInputPartial {
    fn from(params: CreateUserParams) -> Self {
        Self {
            username: params.username.into(),
            error_report: Report::default(),
        }
    }
}

impl ErrorTemplate for UsernameInputPartial {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
    }

    fn with_report(&mut self, report: Report) {
        self.error_report = report;
    }
}

#[derive(Clone, Debug, Default, Template)]
#[template(path = "users/partials/email_input.html")]
pub struct EmailAddressInputPartial {
    pub email: String,
    pub error_report: Report,
}

impl From<CreateUserParams> for EmailAddressInputPartial {
    fn from(params: CreateUserParams) -> Self {
        Self {
            email: params.email.into(),
            error_report: Report::default(),
        }
    }
}

impl ErrorTemplate for EmailAddressInputPartial {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
    }

    fn with_report(&mut self, report: Report) {
        self.error_report = report;
    }
}

#[derive(Clone, Debug, Template)]
#[template(path = "users/partials/password_input.html")]
pub struct PasswordInputPartial {
    pub password: Secret<String>,
    pub error_report: Report,
}

impl Default for PasswordInputPartial {
    fn default() -> Self {
        Self {
            password: Secret::new(String::default()),
            error_report: Report::default(),
        }
    }
}

impl From<CreateUserParams> for PasswordInputPartial {
    fn from(params: CreateUserParams) -> Self {
        Self {
            password: params.password.into(),
            error_report: Report::default(),
        }
    }
}

impl ErrorTemplate for PasswordInputPartial {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
    }

    fn with_report(&mut self, report: Report) {
        self.error_report = report;
    }
}
