use crate::{auth::AuthenticatedUser, models::paste::Paste};
use askama_axum::Template;

#[derive(Template)]
#[template(path = "pastes/new.html")]
pub struct NewPastesTemplate {
    pub current_user: Option<AuthenticatedUser>,
}

#[derive(Template)]
#[template(path = "pastes/index.html")]
pub struct IndexPastesTemplate {
    pub current_user: Option<AuthenticatedUser>,
    pub pastes: Vec<Paste>,
}

#[derive(Template)]
#[template(path = "pastes/show.html")]
pub struct ShowPastesTemplate {
    pub current_user: Option<AuthenticatedUser>,
    pub paste: Paste,
}
