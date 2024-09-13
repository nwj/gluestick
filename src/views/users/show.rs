use crate::helpers::pagination_helper::CursorPaginationResponse;
use crate::helpers::view_helper::filters;
use crate::models::paste::Paste;
use crate::models::session::Session;
use crate::models::user::User;
use askama_axum::Template;

#[derive(Template)]
#[template(path = "users/show.html")]
pub struct ShowPage {
    pub session: Option<Session>,
    pub user: User,
    pub paste_html_pairs: Vec<(Paste, Option<String>)>,
    pub pagination: CursorPaginationResponse,
}
