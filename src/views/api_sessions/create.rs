use crate::models::api_session::UnhashedKey;
use askama_axum::Template;
use secrecy::ExposeSecret;
use uuid::Uuid;

#[derive(Template)]
#[template(path = "api_sessions/create.html")]
pub struct CreatePage {
    pub unhashed_key: UnhashedKey,
    pub api_key_id: Uuid,
}
