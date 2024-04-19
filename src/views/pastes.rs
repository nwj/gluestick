use crate::Paste;
use askama_axum::Template;

#[derive(Template)]
#[template(path = "pastes_new.html")]
pub struct NewPastesTemplate {}

#[derive(Template)]
#[template(path = "pastes_index.html")]
pub struct IndexPastesTemplate {
    pub pastes: Vec<Paste>,
}
