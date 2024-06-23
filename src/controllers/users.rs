use crate::controllers::prelude::*;
use crate::db::Database;
use crate::models::invite_code::InviteCode;
use crate::models::session::{Session, SessionToken};
use crate::models::user::{Password, User};
use crate::views::users::{NewUsersTemplate, ShowUsersTemplate};
use axum::body::Body;
use axum::extract::{Form, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use secrecy::ExposeSecret;
use serde::Deserialize;
use validator::Validate;

pub async fn new() -> NewUsersTemplate {
    NewUsersTemplate { session: None }
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: Password,
    pub invite_code: String,
}

pub async fn create(
    State(db): State<Database>,
    Form(input): Form<CreateUser>,
) -> Result<impl IntoResponse> {
    if let Some(invite_code) = InviteCode::find(&db, input.invite_code).await? {
        // This is called here, rather than by the model (i.e. whenever User is constructed)
        // because we don't want to validate password all the time. For instance, during login,
        // it's good not to enforce this validation, since we are happy to let password guessing
        // attacks try various passwords that we wouldn't actually accept at signup
        input
            .password
            .validate()
            .map_err(|e| Error::BadRequest(Box::new(e)))?;

        let user = User::new(input.username, input.email, &input.password)?;
        user.clone().insert(&db).await?;

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
            .map_err(|e| Error::InternalServerError(Box::new(e)))?;

        Session::new(&token, user).insert(&db).await?;
        invite_code.delete(&db).await?;

        Ok(response)
    } else {
        Err(Error::Unauthorized)
    }
}

pub async fn show(session: Session) -> Result<impl IntoResponse> {
    let session = Some(session);
    Ok(ShowUsersTemplate { session })
}
