use crate::models::{paste::Paste, user::User};
use askama_axum::Template;

#[derive(Template)]
#[template(path = "pastes/new.html")]
pub struct NewPastesTemplate {
    pub current_user: Option<User>,
}

#[derive(Template)]
#[template(path = "pastes/index.html")]
pub struct IndexPastesTemplate {
    pub current_user: Option<User>,
    pub pastes: Vec<Paste>,
}

#[derive(Template)]
#[template(path = "pastes/show.html")]
pub struct ShowPastesTemplate {
    pub current_user: Option<User>,
    pub paste: Paste,
}
