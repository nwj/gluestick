use crate::models::session::Session;
use askama_axum::Template;

#[derive(Template)]
#[template(path = "users/new.html")]
pub struct NewUsersTemplate {
    pub session: Option<()>,
}

#[derive(Template)]
#[template(path = "users/show.html")]
pub struct ShowUsersTemplate {
    pub session: Option<Session>,
}
