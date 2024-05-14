use crate::{auth::AuthenticatedUser, models::paste::Paste};
use askama_axum::Template;

#[derive(Template)]
#[template(path = "pastes_new.html")]
pub struct NewPastesTemplate {
    pub optional_user: Option<AuthenticatedUser>,
}

#[derive(Template)]
#[template(path = "pastes_index.html")]
pub struct IndexPastesTemplate {
    pub optional_user: Option<AuthenticatedUser>,
    pub pastes: Vec<Paste>,
}

#[derive(Template)]
#[template(path = "pastes_show.html")]
pub struct ShowPastesTemplate {
    pub optional_user: Option<AuthenticatedUser>,
    pub paste: Paste,
}
