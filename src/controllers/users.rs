use crate::controllers::prelude::*;
use crate::db::Database;
use crate::helpers::pagination::{CursorPaginationParams, CursorPaginationResponse};
use crate::models::api_session::ApiKey;
use crate::models::invite_code::InviteCode;
use crate::models::paste::Paste;
use crate::models::prelude::Error as ModelsError;
use crate::models::session::{Session, SessionToken, SESSION_COOKIE_NAME};
use crate::models::user::{EmailAddress, UnhashedPassword, User, Username};
use crate::views::users::{
    ChangePasswordFormPartial, EmailAddressInputPartial, NewUsersTemplate, PasswordInputPartial,
    SettingsTemplate, ShowUsersTemplate, UsernameInputPartial,
};
use axum::body::Body;
use axum::extract::{Form, State};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

pub async fn new() -> NewUsersTemplate {
    NewUsersTemplate::default()
}

#[derive(Clone, Deserialize)]
pub struct CreateUserParams {
    pub username: String,
    pub email: String,
    pub password: Secret<String>,
    pub invite_code: String,
}

pub async fn create(
    State(db): State<Database>,
    Form(params): Form<CreateUserParams>,
) -> Result<impl IntoResponse> {
    let mut error_template: NewUsersTemplate = params.clone().into();

    let username_result = Username::try_from(&params.username);
    if let Err(ModelsError::Parse(ref msg)) = username_result {
        error_template.username_error_message = Some(msg.into());
    }
    let email_result = EmailAddress::try_from(&params.email);
    if let Err(ModelsError::Parse(ref msg)) = email_result {
        error_template.email_error_message = Some(msg.into());
    }
    let password_result = UnhashedPassword::try_from(params.password.clone());
    if let Err(ModelsError::Parse(ref msg)) = password_result {
        error_template.password_error_message = Some(msg.into());
    }

    if let Ok(ref username) = username_result {
        if User::find_by_username(&db, username.clone())
            .await?
            .is_some()
        {
            error_template.username_error_message = Some("Username is already taken".into());
        }
    }
    if let Ok(ref email) = email_result {
        if User::find_by_email(&db, email.clone()).await?.is_some() {
            error_template.email_error_message = Some("Email is already taken".into());
        }
    }

    if error_template.username_error_message.is_some()
        || error_template.email_error_message.is_some()
        || error_template.password_error_message.is_some()
    {
        return Err(Error::Unprocessable(Box::new(error_template)));
    }

    let Some(invite_code) = InviteCode::find(&db, &params.invite_code).await? else {
        error_template.invite_code_error_message = Some("Invalid invite code".into());
        return Err(Error::Unprocessable(Box::new(error_template)));
    };

    let (username, email, password) = (username_result?, email_result?, password_result?);
    let user: User = User::new(username, email, password)?;
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
        .map_err(|e| Error::InternalServerError {
            session: None,
            source: Box::new(e),
        })?;
    hashed_token.insert(&db).await?;

    invite_code.delete(&db).await?;

    Ok(response)
}

pub async fn show(
    session: Option<Session>,
    State(db): State<Database>,
    Path(username): Path<String>,
    Query(pagination_params): Query<CursorPaginationParams>,
) -> Result<impl IntoResponse> {
    let username = Username::try_from(&username).map_err(|_| Error::NotFound(session.clone()))?;

    match User::find_by_username(&db, username).await? {
        Some(user) => {
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
        None => Err(Error::NotFound(session)),
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

#[derive(Clone, Deserialize)]
pub struct ChangePasswordParams {
    pub old_password: Secret<String>,
    pub new_password: Secret<String>,
    pub new_password_confirm: Secret<String>,
}

pub async fn change_password(
    session: Session,
    State(db): State<Database>,
    Form(params): Form<ChangePasswordParams>,
) -> Result<impl IntoResponse> {
    let new_password = UnhashedPassword::try_from(params.new_password.clone()).map_err(|e| {
        to_validation_error(Some(session.clone()), e, |msg| ChangePasswordFormPartial {
            new_password_error_message: Some(msg.to_string()),
            ..params.clone().into()
        })
    })?;

    if params.new_password.expose_secret() != params.new_password_confirm.expose_secret() {
        Err(Error::Unprocessable(Box::new(ChangePasswordFormPartial {
            new_password_error_message: Some(
                "New password and password confirmation do not match".into(),
            ),
            ..params.clone().into()
        })))?;
    }

    let old_password = UnhashedPassword::try_from(params.old_password.clone()).map_err(|e| {
        to_validation_error(Some(session.clone()), e, |_| ChangePasswordFormPartial {
            old_password_error_message: Some("Incorrect password".into()),
            ..params.clone().into()
        })
    })?;

    session.user.verify_password(&old_password).map_err(|e| {
        to_validation_error(Some(session.clone()), e, |_| ChangePasswordFormPartial {
            old_password_error_message: Some("Incorrect password".into()),
            ..params.into()
        })
    })?;

    session.user.update_password(&db, new_password).await?;

    Ok(ChangePasswordFormPartial {
        show_success_message: true,
        ..Default::default()
    })
}

pub async fn validate_username(
    session: Option<Session>,
    State(db): State<Database>,
    Form(params): Form<CreateUserParams>,
) -> Result<impl IntoResponse> {
    let username = Username::try_from(&params.username).map_err(|e| {
        to_validation_error(session, e, |msg| UsernameInputPartial {
            username_error_message: Some(msg.into()),
            ..params.clone().into()
        })
    })?;

    if User::find_by_username(&db, username).await?.is_some() {
        Err(Error::Unprocessable(Box::new(UsernameInputPartial {
            username_error_message: Some("Username is already taken".into()),
            ..params.clone().into()
        })))?;
    }

    let template: UsernameInputPartial = params.into();
    Ok(template)
}

pub async fn validate_email(
    session: Option<Session>,
    State(db): State<Database>,
    Form(params): Form<CreateUserParams>,
) -> Result<impl IntoResponse> {
    let email = EmailAddress::try_from(&params.email).map_err(|e| {
        to_validation_error(session, e, |msg| EmailAddressInputPartial {
            email_error_message: Some(msg.into()),
            ..params.clone().into()
        })
    })?;

    if User::find_by_email(&db, email).await?.is_some() {
        Err(Error::Unprocessable(Box::new(EmailAddressInputPartial {
            email_error_message: Some("Email is already taken".into()),
            ..params.clone().into()
        })))?;
    }

    let template: EmailAddressInputPartial = params.into();
    Ok(template)
}

pub async fn validate_password(
    session: Option<Session>,
    Form(params): Form<CreateUserParams>,
) -> Result<impl IntoResponse> {
    let _password = UnhashedPassword::try_from(params.password.clone()).map_err(|e| {
        to_validation_error(session, e, |msg| PasswordInputPartial {
            password_error_message: Some(msg.into()),
            ..params.clone().into()
        })
    })?;

    let template: PasswordInputPartial = params.into();
    Ok(template)
}
