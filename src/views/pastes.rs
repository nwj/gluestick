use crate::models::paste::Paste;
use askama_axum::Template;

#[derive(Template)]
#[template(path = "pastes_new.html")]
pub struct NewPastesTemplate {}

#[derive(Template)]
#[template(path = "pastes_index.html")]
pub struct IndexPastesTemplate {
    pub pastes: Vec<Paste>,
}

#[derive(Template)]
#[template(path = "pastes_show.html")]
pub struct ShowPastesTemplate {
    pub maybe_paste: Option<Paste>,
}
