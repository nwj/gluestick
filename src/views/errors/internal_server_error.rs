use crate::models::session::Session;
use askama::Template;

#[derive(Template)]
#[template(path = "errors/500.html")]
pub struct InternalServerErrorPage {
    pub session: Option<Session>,
}
