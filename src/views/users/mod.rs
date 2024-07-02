use crate::controllers::prelude::ErrorTemplate;
use crate::models::session::Session;
use crate::params::prelude::Report;
use crate::params::users::CreateUserParams;
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
    pub validation_report: Report,
}

impl NewUsersTemplate {
    pub fn from_params(params: CreateUserParams) -> Self {
        Self {
            session: None,
            username: params.username.into(),
            email: params.email.into(),
            password: params.password.into_inner().expose_secret().to_string(),
            invite_code: params.invite_code.into(),
            validation_report: Report::default(),
        }
    }
}

impl ErrorTemplate for NewUsersTemplate {
    fn render_template(&self) -> askama::Result<String> {
        self.render()
    }

    fn with_report(&mut self, report: Report) {
        self.validation_report = report;
    }
}

#[derive(Template)]
#[template(path = "users/show.html")]
pub struct ShowUsersTemplate {
    pub session: Option<Session>,
}
