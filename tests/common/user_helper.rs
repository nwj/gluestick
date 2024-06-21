use crate::common::client::TestClient;
use crate::common::rand_helper;
use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct TestUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Default, Clone)]
pub struct TestUserBuilder {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

impl TestUser {
    pub fn builder() -> TestUserBuilder {
        TestUserBuilder::new()
    }

    pub async fn persist(self, client: &TestClient, invite_code: String) -> Result<Self> {
        client.signup().post(invite_code, &self).await?;
        // logout so that we don't leave a persisted session on the client
        client.logout().delete().await?;
        Ok(self)
    }

    pub async fn persist_with_session(
        self,
        client: &TestClient,
        invite_code: String,
    ) -> Result<Self> {
        client.signup().post(invite_code, &self).await?;
        Ok(self)
    }
}

impl TestUserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        let _ = self.username.insert(username.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        let _ = self.email.insert(email.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        let _ = self.password.insert(password.into());
        self
    }

    pub fn random_username(self) -> Result<Self> {
        Ok(self.username(rand_helper::random_alphanumeric_string(3..=25)?))
    }

    pub fn random_email(self) -> Result<Self> {
        Ok(self.email(rand_helper::random_email(3..=35)?))
    }

    pub fn random_password(self) -> Result<Self> {
        Ok(self.password(rand_helper::random_string(8..=20)?))
    }

    pub fn random(self) -> Result<Self> {
        Ok(self.random_username()?.random_email()?.random_password()?)
    }

    pub fn build(self) -> TestUser {
        let username = self.username.clone().unwrap_or("jmanderley".into());
        let email = self.email.clone().unwrap_or("jmanderley@unatco.gov".into());
        let password = self.password.clone().unwrap_or("knight_killer".into());

        TestUser {
            username,
            email,
            password,
        }
    }
}
