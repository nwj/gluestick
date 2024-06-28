use crate::controllers::prelude::*;
use crate::db::Database;
use crate::models::invite_code::InviteCode;
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
    match params.clone().validate() {
        Ok(validated_params) => {
            if let Some(invite_code) =
                InviteCode::find(&db, validated_params.0.invite_code.0.clone()).await?
            {
                let user: User = validated_params.try_into()?;
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
            } else {
                Err(Error::Unauthorized)
            }
        }
        Err(report) => {
            let params = params.into_inner();
            let template = NewUsersTemplate {
                session: None,
                username: params.username.into_inner(),
                email: params.email.into_inner(),
                password: params.password.expose_secret().to_string(),
                invite_code: params.invite_code.into_inner(),
                validation_report: report,
            };
            Ok(template.into_response())
        }
    }
}

pub async fn show(session: Session) -> Result<impl IntoResponse> {
    let session = Some(session);
    Ok(ShowUsersTemplate { session })
}
