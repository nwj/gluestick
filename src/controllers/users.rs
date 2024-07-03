use crate::controllers::prelude::*;
use crate::db::Database;
use crate::models::session::{Session, SessionToken, SESSION_COOKIE_NAME};
use crate::models::user::User;
use crate::params::prelude::{Validate, Verify};
use crate::params::users::CreateUserParams;
use crate::views::users::{
    EmailAddressInputPartial, NewUsersTemplate, PasswordInputPartial, ShowUsersTemplate,
    UsernameInputPartial,
};
use axum::body::Body;
use axum::extract::{Form, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use secrecy::ExposeSecret;

pub async fn new() -> NewUsersTemplate {
    NewUsersTemplate::default()
}

pub async fn create(
    State(db): State<Database>,
    Form(params): Form<CreateUserParams>,
) -> Result<impl IntoResponse> {
    let error_template: NewUsersTemplate = params.clone().into();

    params
        .validate()
        .map_err(|e| handle_params_error(e, error_template.clone()))?;
    let invite_code = params
        .clone()
        .verify(&db)
        .await
        .map_err(|e| handle_params_error(e, error_template))?;

    let user: User = params.try_into()?;
    user.clone().insert(&db).await?;

    let token = SessionToken::generate();
    let response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/")
        .header(
            "Set-Cookie",
            format!(
                "{}={}; Max-Age=999999; Secure; HttpOnly",
                SESSION_COOKIE_NAME,
                &token.expose_secret()
            ),
        )
        .body(Body::empty())
        .map_err(|e| Error::InternalServerError(Box::new(e)))?;

    Session::new(&token, user).insert(&db).await?;
    invite_code.delete(&db).await?;

    Ok(response)
}

pub async fn show(session: Session) -> Result<impl IntoResponse> {
    let session = Some(session);
    Ok(ShowUsersTemplate { session })
}

pub async fn validate_username(
    State(db): State<Database>,
    Form(params): Form<CreateUserParams>,
) -> Result<impl IntoResponse> {
    let username = params.username.clone();
    let template: UsernameInputPartial = params.into();

    username
        .validate()
        .map_err(|e| handle_params_error(e, template.clone()))?;

    username
        .verify(&db)
        .await
        .map_err(|e| handle_params_error(e, template.clone()))?;

    Ok(template)
}

pub async fn validate_email(
    State(db): State<Database>,
    Form(params): Form<CreateUserParams>,
) -> Result<impl IntoResponse> {
    let email = params.email.clone();
    let template: EmailAddressInputPartial = params.into();

    email
        .validate()
        .map_err(|e| handle_params_error(e, template.clone()))?;
    email
        .verify(&db)
        .await
        .map_err(|e| handle_params_error(e, template.clone()))?;

    Ok(template)
}
pub async fn validate_password(Form(params): Form<CreateUserParams>) -> Result<impl IntoResponse> {
    let password = params.password.clone();
    let template: PasswordInputPartial = params.into();

    password
        .validate()
        .map_err(|e| handle_params_error(e, template.clone()))?;

    Ok(template)
}
