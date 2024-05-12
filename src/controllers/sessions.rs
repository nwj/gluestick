use crate::{controllers, db::Database, models::user::User, views::sessions::NewSessionsTemplate};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect},
};
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde::Deserialize;
use validator::Validate;

pub async fn new() -> NewSessionsTemplate {
    NewSessionsTemplate {}
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

    Ok(Redirect::to("/").into_response())
}
