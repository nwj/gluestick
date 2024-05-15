use crate::{
    auth::AuthenticatedUser,
    controllers,
    db::Database,
    models::paste::Paste,
    validators,
    views::pastes::{IndexPastesTemplate, NewPastesTemplate, ShowPastesTemplate},
};
use axum::{
    extract::{Form, Path, State},
    http::{header::HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

pub async fn index(
    current_user: Option<AuthenticatedUser>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::Error> {
    let pastes = Paste::all(&db)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;
    Ok(IndexPastesTemplate {
        current_user,
        pastes,
    })
}

pub async fn new(user: AuthenticatedUser) -> NewPastesTemplate {
    let current_user = Some(user);
    NewPastesTemplate { current_user }
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreateFormInput {
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub title: String,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub description: String,
    #[validate(custom(function = "validators::not_empty_when_trimmed"))]
    pub body: String,
}

pub async fn create(
    State(db): State<Database>,
    Form(input): Form<CreateFormInput>,
) -> Result<impl IntoResponse, controllers::Error> {
    input.validate()?;
    let id = Paste::insert(&db, input.title, input.description, input.body)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;
    Ok(Redirect::to(format!("/pastes/{}", id).as_str()).into_response())
}

pub async fn show(
    Path(id): Path<Uuid>,
    current_user: Option<AuthenticatedUser>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::Error> {
    match Paste::find(&db, id)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?
    {
        Some(paste) => Ok((
            StatusCode::OK,
            ShowPastesTemplate {
                current_user,
                paste,
            },
        )),
        None => Err(controllers::Error::NotFound),
    }
}

pub async fn destroy(
    Path(id): Path<Uuid>,
    State(db): State<Database>,
) -> Result<impl IntoResponse, controllers::Error> {
    Paste::delete(&db, id)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/pastes"));
    Ok(headers)
}
