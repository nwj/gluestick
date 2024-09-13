use crate::models::session::Session;
use askama::Template;

#[derive(Template)]
#[template(path = "errors/403.html")]
pub struct ForbiddenPage {
    pub session: Option<Session>,
}
