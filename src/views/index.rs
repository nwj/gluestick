use crate::models::session::Session;
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexPage {
    pub session: Option<Session>,
}
