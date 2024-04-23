use askama::Template;

pub mod pastes;

#[derive(Template)]
#[template(path = "404.html")]
pub struct NotFoundTemplate {}
