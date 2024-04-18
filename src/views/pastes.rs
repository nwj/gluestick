use crate::Paste;
use askama_axum::Template;

#[derive(Template)]
#[template(path = "new_paste.html")]
pub struct NewPasteTemplate {}

#[derive(Template)]
#[template(path = "pastes_index.html")]
pub struct PastesIndexTemplate {
    pub pastes: Vec<Paste>,
}
