use crate::controllers::prelude::*;
use crate::db::Database;
use crate::models::session::{Session, SessionToken, SESSION_COOKIE_NAME};
use crate::models::user::{EmailAddress, UnhashedPassword, User};
use crate::views::sessions::NewSessionsTemplate;
use axum::body::Body;
use axum::extract::{Form, State};
use axum::http::{header::HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

pub async fn new() -> NewSessionsTemplate {
    NewSessionsTemplate::default()
}

#[derive(Clone, Deserialize)]
pub struct CreateSessionParams {
    pub email: String,
    pub password: Secret<String>,
}

pub async fn create(
    State(db): State<Database>,
    Form(params): Form<CreateSessionParams>,
) -> Result<impl IntoResponse> {
    let email = EmailAddress::try_from(&params.email).map_err(|e| {
        to_validation_error(e, |_| NewSessionsTemplate {
            error_message: Some("Incorrect email or password".into()),
            ..params.clone().into()
        })
    })?;

    let password = UnhashedPassword::try_from(params.password.clone()).map_err(|e| {
        to_validation_error(e, |_| NewSessionsTemplate {
            error_message: Some("Incorrect email or password".into()),
            ..params.clone().into()
        })
    })?;

    let user = match User::find_by_email2(&db, email).await? {
        Some(user) if user.verify_password2(&password).is_ok() => user,
        _ => Err(Error::Validation2(Box::new(NewSessionsTemplate {
            error_message: Some("Incorrect email or password".into()),
            ..params.into()
        })))?,
    };

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
