use askama::Template;

pub mod pastes;
pub mod sessions;
pub mod users;

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
