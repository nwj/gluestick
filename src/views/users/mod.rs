use crate::models::session::Session;
use crate::params::prelude::Report;
use askama_axum::Template;

#[derive(Default, Template)]
#[template(path = "users/new.html")]
pub struct NewUsersTemplate {
    pub session: Option<()>,
    pub username: String,
    pub email: String,
    pub password: String,
    pub invite_code: String,
    pub validation_report: Report,
}

#[derive(Template)]
#[template(path = "users/show.html")]
pub struct ShowUsersTemplate {
    pub session: Option<Session>,
}
