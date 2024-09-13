use crate::models::session::Session;
use askama::Template;

#[derive(Template)]
#[template(path = "errors/404.html")]
pub struct NotFoundPage {
    pub session: Option<Session>,
}
