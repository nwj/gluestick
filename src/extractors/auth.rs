use crate::controllers::api::prelude::Error as ApiControllerError;
use crate::controllers::prelude::Error as ControllerError;
use crate::db::Database;
use crate::models::api_session::{ApiKey, ApiSession, API_KEY_HEADER_NAME};
use crate::models::session::{Session, UnhashedToken, SESSION_COOKIE_NAME};
use crate::models::user::User;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::{async_trait, RequestPartsExt};
use axum_extra::extract::CookieJar;

#[async_trait]
impl<S> FromRequestParts<S> for Session
where
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ControllerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db = parts
            .extract_with_state::<Database, _>(state)
            .await
            .map_err(|e| ControllerError::InternalServerError(Box::new(e)))?;

        let cookie = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| ControllerError::Unauthorized)?
            .get(SESSION_COOKIE_NAME)
            .ok_or(ControllerError::Unauthorized)?
            .to_owned();

        let token =
            UnhashedToken::try_from(cookie.value()).map_err(|_| ControllerError::Unauthorized)?;

        let maybe_session = Session::find_by_unhashed_token(&db, &token).await?;

        match maybe_session {
            Some(session) => Ok(session),
            None => Err(ControllerError::Unauthorized),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiSession
where
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiControllerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db = parts
            .extract_with_state::<Database, _>(state)
            .await
            .map_err(|e| ApiControllerError::InternalServerError(Box::new(e)))?;

        let header = parts
            .headers
            .get(API_KEY_HEADER_NAME)
            .ok_or(ApiControllerError::Unauthorized)?
            .to_str()
            .map_err(|_| ApiControllerError::Unauthorized)?;

        let api_key = ApiKey::try_from(header).map_err(|_| ApiControllerError::Unauthorized)?;

        let optional_user = User::find_by_api_key(&db, &api_key).await?;

        match optional_user {
            Some(user) => Ok(ApiSession::new(&api_key, user)),
            None => Err(ApiControllerError::Unauthorized),
        }
    }
}
