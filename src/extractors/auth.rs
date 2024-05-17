use crate::{
    controllers,
    db::Database,
    models::{
        session::{Session, SessionToken},
        user::User,
    },
};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::extract::CookieJar;

#[async_trait]
impl<S> FromRequestParts<S> for Session
where
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = controllers::Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db = parts
            .extract_with_state::<Database, _>(state)
            .await
            .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

        let cookie_jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| controllers::Error::Unauthorized)?;
        let cookie = cookie_jar
            .get("session_token")
            .ok_or(controllers::Error::Unauthorized)?;
        let token =
            SessionToken::parse(cookie.value()).map_err(|_| controllers::Error::Unauthorized)?;

        let optional_user = User::find_by_session_token(&db, token.clone())
            .await
            .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

        match optional_user {
            Some(user) => Ok(Session { token, user }),
            None => Err(controllers::Error::Unauthorized),
        }
    }
}
