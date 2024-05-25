use crate::{
    controllers,
    db::Database,
    models::{
        session::{Session, SessionToken},
        user::{Password, User},
    },
    views::sessions::NewSessionsTemplate,
};
use axum::{
    body::Body,
    extract::{Form, State},
    http::{header::HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use secrecy::ExposeSecret;
use serde::Deserialize;
use validator::Validate;

pub async fn new() -> NewSessionsTemplate {
    NewSessionsTemplate { session: None }
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreateSession {
    pub email: String,
    pub password: Password,
}

pub async fn create(
    State(db): State<Database>,
    Form(input): Form<CreateSession>,
) -> controllers::Result<impl IntoResponse> {
    let Some(user) = User::find_by_email(&db, input.email).await? else {
        return Err(controllers::Error::Unauthorized);
    };

    user.verify_password(input.password)
        .map_err(|_| controllers::Error::Unauthorized)?;

    let token = SessionToken::generate();

    let response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/")
        .header(
            "Set-Cookie",
            format!(
                "session_token={}; Max-Age=999999; Secure; HttpOnly",
                &token.expose_secret()
            ),
        )
        .body(Body::empty())
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

    Session::new(token, user).insert(&db).await?;

    Ok(response)
}

pub async fn delete(
    session: Session,
    State(db): State<Database>,
) -> controllers::Result<impl IntoResponse> {
    session.user.delete_sessions(&db).await?;

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/login"));
    Ok(headers)
}
