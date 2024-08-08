use crate::controllers::prelude::ErrorTemplate;
use crate::helpers::pagination::CursorPaginationResponse;
use crate::models::api_session::ApiKey;
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::User;
use crate::params::prelude::Report;
use crate::params::users::CreateUserParams;
use crate::views::filters;
use askama_axum::Template;
use secrecy::ExposeSecret;

#[derive(Clone, Debug, Default, Template)]
#[template(path = "users/new.html")]
pub struct NewUsersTemplate {
    pub session: Option<()>,
    pub username: String,
    pub email: String,
    pub password: String,
    pub invite_code: String,
    pub error_report: Report,
}

impl From<CreateUserParams> for NewUsersTemplate {
    fn from(params: CreateUserParams) -> Self {
        Self {
            session: None,
            username: params.username.into(),
            email: params.email.into(),
            password: params.password.expose_secret().to_string(),
            invite_code: params.invite_code.into(),
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

#[derive(Template)]
#[template(path = "users/settings.html")]
pub struct SettingsTemplate {
    pub session: Option<Session>,
    pub api_keys: Vec<ApiKey>,
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

#[derive(Clone, Debug, Default, Template)]
#[template(path = "users/partials/password_input.html")]
pub struct PasswordInputPartial {
    pub password: String,
    pub error_report: Report,
}

impl From<CreateUserParams> for PasswordInputPartial {
    fn from(params: CreateUserParams) -> Self {
        Self {
            password: params.password.expose_secret().into(),
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
