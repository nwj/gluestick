use crate::{
    controllers, db::Database, models::user::User, validators, views::users::NewUsersTemplate,
};
use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect},
};
use secrecy::Secret;
use serde::Deserialize;
use validator::Validate;

pub async fn new() -> NewUsersTemplate {
    NewUsersTemplate {
        current_user: None,
    }
}

#[derive(Deserialize, Debug, Validate)]
pub struct CreateUser {
    #[validate(custom(function = "validators::is_valid_username"))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub password: Secret<String>,
}

pub async fn create(
    State(db): State<Database>,
    Form(input): Form<CreateUser>,
) -> Result<impl IntoResponse, controllers::Error> {
    input.validate()?;
    User::insert(&db, input.username, input.email, input.password)
        .await
        .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;
    Ok(Redirect::to("/").into_response())
}
