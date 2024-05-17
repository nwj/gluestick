use crate::models::{paste::Paste, session::Session};
use askama_axum::Template;

#[derive(Template)]
#[template(path = "pastes/new.html")]
pub struct NewPastesTemplate {
    pub session: Option<Session>,
}

#[derive(Template)]
#[template(path = "pastes/index.html")]
pub struct IndexPastesTemplate {
    pub session: Option<Session>,
    pub pastes: Vec<Paste>,
}

#[derive(Template)]
#[template(path = "pastes/show.html")]
pub struct ShowPastesTemplate {
    pub session: Option<Session>,
    pub paste: Paste,
}
