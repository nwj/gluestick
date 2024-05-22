use crate::{
    controllers,
    db::Database,
    models::{
        session::{Session, SessionToken},
        user::{Password, User},
    },
    validators,
    views::users::{NewUsersTemplate, ShowUsersTemplate},
};
use axum::{
    body::Body,
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use secrecy::ExposeSecret;
use serde::Deserialize;
use validator::Validate;

pub async fn new() -> NewUsersTemplate {
    NewUsersTemplate { session: None }
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreateUser {
    #[validate(custom(function = "validators::is_valid_username"))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub password: Password,
}

pub async fn create(
    State(db): State<Database>,
    Form(input): Form<CreateUser>,
) -> controllers::Result<impl IntoResponse> {
    input.validate()?;
    let user = User::new(input.username, input.email, input.password)
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;
    user.clone()
        .insert(&db)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

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

    Session::new(token, user)
        .insert(&db)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

    Ok(response)
}

pub async fn show(session: Session) -> controllers::Result<impl IntoResponse> {
    let session = Some(session);
    Ok(ShowUsersTemplate { session })
}
