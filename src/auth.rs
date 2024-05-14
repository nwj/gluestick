use crate::{controllers, db::Database, models::user::User};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::extract::CookieJar;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use secrecy::{ExposeSecret, Secret};
use sha2::{Digest, Sha256};
use std::num::ParseIntError;

#[derive(Debug)]
pub struct AuthenticatedUser(pub User);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
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
        let Some(cookie) = cookie_jar.get("session_token") else {
            return Err(controllers::Error::Unauthorized);
        };
        let session_token =
            SessionToken::parse(cookie.value()).map_err(|_| controllers::Error::Unauthorized)?;

        let optional_user = User::find_by_session_token(&db, session_token)
            .await
            .map_err(|e| controllers::Error::InternalServerError(Box::new(e)))?;

        match optional_user {
            Some(user) => Ok(AuthenticatedUser(user)),
            None => Err(controllers::Error::Unauthorized),
        }
    }
}

#[derive(Clone)]
pub struct SessionToken(Secret<String>);

impl SessionToken {
    pub fn generate() -> Self {
        // The OWASP checklist for session tokens:
        // - has a size of at least 128-bits: ours is 128-bits
        // - contains at least 64-bits of entropy: use of ChaCha20 seeded by the OS should ensure this
        // - must be unique: uniqueness is statistically likely here, but enforced elsewhere by database constraint
        //
        // See: https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
        let mut rng = ChaCha20Rng::from_entropy();
        SessionToken(Secret::new(format!("{:032x}", rng.gen::<u128>())))
    }

    pub fn to_hash(&self) -> HashedSessionToken {
        HashedSessionToken(Secret::new(
            Sha256::digest(self.expose_secret().as_bytes()).to_vec(),
        ))
    }

    pub fn parse(s: impl AsRef<str>) -> Result<Self, ParseIntError> {
        let s = s.as_ref();
        u128::from_str_radix(s, 16)?;
        Ok(SessionToken(Secret::new(s.to_string())))
    }
}

impl ExposeSecret<String> for SessionToken {
    fn expose_secret(&self) -> &String {
        self.0.expose_secret()
    }
}

pub struct HashedSessionToken(Secret<Vec<u8>>);

impl ExposeSecret<Vec<u8>> for HashedSessionToken {
    fn expose_secret(&self) -> &Vec<u8> {
        self.0.expose_secret()
    }
}
