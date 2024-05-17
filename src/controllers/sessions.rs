use crate::{
    controllers,
    db::Database,
    models::{session::SessionToken, user::User},
    views::sessions::NewSessionsTemplate,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    body::Body,
    extract::{Form, State},
    http::{header::HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde::Deserialize;
use validator::Validate;

pub async fn new() -> NewSessionsTemplate {
    NewSessionsTemplate { current_user: None }
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreateSession {
    #[validate(email)]
    pub email: String,
    pub password: Secret<String>,
}

pub async fn create(
    State(db): State<Database>,
    Form(input): Form<CreateSession>,
) -> Result<impl IntoResponse, controllers::Error> {
    input.validate()?;

    let Some(user) = User::find_by_email(&db, input.email)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?
    else {
        return Err(controllers::Error::Unauthorized);
    };

    if let Err(_e) = Argon2::default().verify_password(
        input.password.expose_secret().as_bytes(),
        &PasswordHash::new(user.password.expose_secret())
            .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?,
    ) {
        return Err(controllers::Error::Unauthorized);
    };

    let session_token = SessionToken::generate();

    let response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/")
        .header(
            "Set-Cookie",
            format!(
                "session_token={}; Max-Age=999999; Secure; HttpOnly",
                session_token.expose_secret()
            ),
        )
        .body(Body::empty())
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

    session_token
        .insert(&db, user.id)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

    Ok(response)
}

pub async fn delete(
    current_user: User,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::Error> {
    SessionToken::delete_by_user_id(&db, current_user.id)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/login"));
    Ok(headers)
}
