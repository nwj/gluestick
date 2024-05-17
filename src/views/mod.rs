use crate::models::user::User;
use askama::Template;

pub mod pastes;
pub mod sessions;
pub mod users;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub current_user: Option<User>,
}

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate {
    pub current_user: Option<()>,
}

#[derive(Template)]
#[template(path = "500.html")]
pub struct InternalServerErrorTemplate {
    pub current_user: Option<()>,
}
