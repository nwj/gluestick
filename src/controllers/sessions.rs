use crate::controllers::prelude::*;
use crate::db::Database;
use crate::models::session::{Session, SessionToken, SESSION_COOKIE_NAME};
use crate::models::user::User;
use crate::params::prelude::Validate;
use crate::params::sessions::CreateSessionParams;
use crate::views::sessions::NewSessionsTemplate;
use axum::body::Body;
use axum::extract::{Form, State};
use axum::http::{header::HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use secrecy::ExposeSecret;

pub async fn new() -> NewSessionsTemplate {
    NewSessionsTemplate { session: None }
}

pub async fn create(
    State(db): State<Database>,
    Form(params): Form<CreateSessionParams>,
) -> Result<impl IntoResponse> {
    params.validate().map_err(|_| Error::Unauthorized)?;

    let Some(user) = User::find_by_email(&db, params.email.into_inner()).await? else {
        return Err(Error::Unauthorized);
    };

    user.verify_password(params.password.into_inner().expose_secret())
        .map_err(|_| Error::Unauthorized)?;

    let token = SessionToken::generate();

    let response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/")
        .header(
            "Set-Cookie",
            format!(
                "{}={}; Max-Age=999999; Secure; HttpOnly; SameSite=Lax",
                SESSION_COOKIE_NAME,
                &token.expose_secret()
            ),
        )
        .body(Body::empty())
        .map_err(|e| Error::InternalServerError(Box::new(e)))?;

    Session::new(&token, user).insert(&db).await?;

    Ok(response)
}

pub async fn delete(session: Session, State(db): State<Database>) -> Result<impl IntoResponse> {
    session.user.delete_sessions(&db).await?;

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/login"));
    Ok(headers)
}
