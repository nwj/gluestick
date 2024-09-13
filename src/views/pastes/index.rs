use crate::helpers::pagination_helper::CursorPaginationResponse;
use crate::helpers::view_helper::filters;
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::Username;
use askama_axum::Template;

#[derive(Template)]
#[template(path = "pastes/index.html")]
pub struct IndexPage {
    pub session: Option<Session>,
    pub paste_username_html_triples: Vec<(Paste, Username, Option<String>)>,
    pub pagination: CursorPaginationResponse,
}
