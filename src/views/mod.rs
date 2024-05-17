use crate::models::session::Session;
use askama::Template;

pub mod pastes;
pub mod sessions;
pub mod users;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub session: Option<Session>,
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate {
    pub session: Option<()>,
}

#[derive(Template)]
#[template(path = "500.html")]
pub struct InternalServerErrorTemplate {
    pub session: Option<()>,
}
