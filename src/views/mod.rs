use askama::Template;

pub mod pastes;

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate {}

#[derive(Template)]
#[template(path = "500.html")]
pub struct InternalServerErrorTemplate {}
