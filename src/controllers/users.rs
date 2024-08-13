use crate::controllers::prelude::*;
use crate::db::Database;
use crate::helpers::pagination::{CursorPaginationParams, CursorPaginationResponse};
use crate::models::api_session::ApiKey;
use crate::models::paste::Paste;
use crate::models::session::{Session, SessionToken, SESSION_COOKIE_NAME};
use crate::models::user::User;
use crate::params::users::{ChangePasswordParams, CreateUserParams, UsernameParam};
use crate::views::users::{
    ChangePasswordFormPartial, EmailAddressInputPartial, NewUsersTemplate, PasswordInputPartial,
    SettingsTemplate, ShowUsersTemplate, UsernameInputPartial,
};
use axum::body::Body;
use axum::extract::{Form, State};
use axum::extract::{Path, Query};
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
        .validate_and_check_if_taken(&db)
        .await
        .map_err(|e| handle_params_error(e, error_template.clone()))?;

    let invite_code = params
        .verify_invite_code(&db)
        .await
        .map_err(|e| handle_params_error(e, error_template))?;

    let user: User = User::new(params.username, params.email, params.password)?;
    let user_id = user.id;
    user.insert(&db).await?;

    let (unhashed_token, hashed_token) = SessionToken::new(user_id);
    let response = Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/new")
        .header(
            "Set-Cookie",
            format!(
                "{}={}; Max-Age=999999; Secure; HttpOnly",
                SESSION_COOKIE_NAME,
                &unhashed_token.expose_secret()
            ),
        )
        .body(Body::empty())
        .map_err(|e| Error::InternalServerError(Box::new(e)))?;
    hashed_token.insert(&db).await?;

    invite_code.delete(&db).await?;

    Ok(response)
}

pub async fn show(
    session: Option<Session>,
    State(db): State<Database>,
    Path(username): Path<UsernameParam>,
    Query(pagination_params): Query<CursorPaginationParams>,
) -> Result<impl IntoResponse> {
    username.validate().map_err(|_| Error::NotFound)?;

    match User::find_by_username(&db, username).await? {
        Some(user) => {
            pagination_params
                .validate()
                .map_err(|e| Error::BadRequest(Box::new(e)))?;

            let mut pastes = if Some(&user) == session.as_ref().map(|s| &s.user) {
                Paste::cursor_paginated_for_user_id_with_secrets(
                    &db,
                    user.id,
                    pagination_params.limit_with_lookahead(),
                    pagination_params.direction(),
                    pagination_params.cursor(),
                )
                .await?
            } else {
                Paste::cursor_paginated_for_user_id(
                    &db,
                    user.id,
                    pagination_params.limit_with_lookahead(),
                    pagination_params.direction(),
                    pagination_params.cursor(),
                )
                .await?
            };
            let pagination_response =
                CursorPaginationResponse::new_with_lookahead(&pagination_params, &mut pastes);
            let mut pairs = Vec::new();
            for paste in pastes {
                let optional_html = paste
                    .syntax_highlight(&db) // This is an n+1 query, but it's fine because our cache is SQLite.
                    .await?;
                pairs.push((paste, optional_html));
            }
            Ok(ShowUsersTemplate {
                session,
                user,
                paste_html_pairs: pairs,
                pagination: pagination_response,
            })
        }
        None => Err(Error::NotFound),
    }
}

pub async fn settings(session: Session, State(db): State<Database>) -> Result<impl IntoResponse> {
    let api_keys = ApiKey::all_for_user_id(&db, session.user.id).await?;
    let session = Some(session);

    Ok(SettingsTemplate {
        session,
        api_keys,
        ..Default::default()
    })
}

pub async fn change_password(
    session: Session,
    State(db): State<Database>,
    Form(params): Form<ChangePasswordParams>,
) -> Result<impl IntoResponse> {
    let error_template: ChangePasswordFormPartial = params.clone().into();

    params
        .validate()
        .map_err(|e| handle_params_error(e, error_template.clone()))?;
    params
        .authenticate(&session.user)
        .map_err(|e| handle_params_error(e, error_template))?;

    session
        .user
        .update_password(&db, params.new_password)
        .await?;

    Ok(ChangePasswordFormPartial {
        show_success_message: true,
        ..Default::default()
    })
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
        .check_if_taken(&db)
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
        .check_if_taken(&db)
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
