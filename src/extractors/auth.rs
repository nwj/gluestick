use crate::controllers::api::prelude::Error as ApiControllerError;
use crate::controllers::prelude::Error as ControllerError;
use crate::db::Database;
use crate::models::api_session::{ApiKey, ApiSession};
use crate::models::session::{Session, SessionToken};
use crate::models::user::User;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::{async_trait, RequestPartsExt};
use axum_extra::extract::CookieJar;

const X_API_KEY: &str = "X-GLUESTICK-API-KEY";

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

        let cookie_jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| ControllerError::Unauthorized)?;
        let cookie = cookie_jar
            .get("session_token")
            .ok_or(ControllerError::Unauthorized)?;
        let token =
            SessionToken::parse(cookie.value()).map_err(|_| ControllerError::Unauthorized)?;

        let optional_user = User::find_by_session_token(&db, token.clone()).await?;

        match optional_user {
            Some(user) => Ok(Session::new(&token, user)),
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

        let header_content = parts
            .headers
            .get(X_API_KEY)
            .ok_or(ApiControllerError::Unauthorized)?
            .to_str()
            .map_err(|_| ApiControllerError::Unauthorized)?;

        let api_key =
            ApiKey::parse(header_content).map_err(|_| ApiControllerError::Unauthorized)?;

        let optional_user = User::find_by_api_key(&db, api_key.clone()).await?;

        match optional_user {
            Some(user) => Ok(ApiSession::new(&api_key, user)),
            None => Err(ApiControllerError::Unauthorized),
        }
    }
}
