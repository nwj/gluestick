use crate::controllers::prelude::*;
use crate::db::Database;
use crate::models::session::{Session, SessionToken, SESSION_COOKIE_NAME};
use crate::models::user::User;
use crate::params::prelude::Unvalidated;
use crate::params::users::CreateUserParams;
use crate::views::users::{NewUsersTemplate, ShowUsersTemplate};
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
    Form(params): Form<Unvalidated<CreateUserParams>>,
) -> Result<impl IntoResponse> {
    let error_template = NewUsersTemplate::from_params(params.clone().into_inner());

    let valid_params = params
        .clone()
        .validate()
        .map_err(|e| handle_params_error(e, error_template.clone()))?;

    let invite_code = valid_params
        .clone()
        .verify(&db)
        .await
        .map_err(|e| handle_params_error(e, error_template))?;

    let user: User = valid_params.try_into()?;
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
