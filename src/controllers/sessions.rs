use crate::controllers::prelude::*;
use crate::db::Database;
use crate::models::session::{Session, SessionToken, SESSION_COOKIE_NAME};
use crate::params::sessions::CreateSessionParams;
use crate::views::sessions::NewSessionsTemplate;
use axum::body::Body;
use axum::extract::{Form, State};
use axum::http::{header::HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use secrecy::ExposeSecret;

pub async fn new() -> NewSessionsTemplate {
    NewSessionsTemplate::default()
}

pub async fn create(
    State(db): State<Database>,
    Form(params): Form<CreateSessionParams>,
) -> Result<impl IntoResponse> {
    let user = params
        .clone()
        .authenticate(&db)
        .await
        .map_err(|e| handle_params_error(e, NewSessionsTemplate::from(params)))?;

    let (unhashed_token, hashed_token) = SessionToken::new(user.id);
    let response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/new")
        .header(
            "Set-Cookie",
            format!(
                "{}={}; Max-Age=999999; Secure; HttpOnly; SameSite=Lax",
                SESSION_COOKIE_NAME,
                &unhashed_token.expose_secret()
            ),
        )
        .body(Body::empty())
        .map_err(|e| Error::InternalServerError(Box::new(e)))?;
    hashed_token.insert(&db).await?;

    Ok(response)
}

pub async fn delete(session: Session, State(db): State<Database>) -> Result<impl IntoResponse> {
    session.user.delete_sessions(&db).await?;

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/"));
    Ok(headers)
}
