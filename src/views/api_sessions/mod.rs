use crate::models::api_session::ApiKey;
use askama_axum::Template;
use secrecy::ExposeSecret;

#[derive(Template)]
#[template(path = "api_sessions/create.html")]
pub struct CreateApiSessionsTemplate {
    pub api_key: ApiKey,
}
