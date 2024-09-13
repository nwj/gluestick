use crate::models::session::Session;
use askama::Template;

#[derive(Template)]
#[template(path = "errors/401.html")]
pub struct UnauthorizedPage {
    pub session: Option<Session>,
}
