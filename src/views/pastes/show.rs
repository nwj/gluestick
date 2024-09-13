use crate::helpers::view_helper::filters;
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::Username;
use askama_axum::Template;

#[derive(Template)]
#[template(path = "pastes/show.html")]
pub struct ShowPage {
    pub session: Option<Session>,
    pub paste: Paste,
    pub username: Username,
    pub syntax_highlighted_html: Option<String>,
}
