use crate::common::mocks::mock_api_key::MockApiKey;
use crate::common::rand_helper;
use crate::common::test_app::TestApp;
use crate::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct MockUser {
    pub id: Option<String>,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone, Default)]
pub struct MockUserBuilder {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

impl MockUser {
    pub fn builder() -> MockUserBuilder {
        MockUserBuilder::new()
    }

    pub async fn seed(mut self, app: &TestApp) -> Result<Self> {
        self.id = Some(Uuid::now_v7().to_string());
        app.seed_user(self.clone()).await?;
        Ok(self)
    }

    pub async fn seed_with_api_key(mut self, app: &TestApp) -> Result<(Self, MockApiKey)> {
        let id = Uuid::now_v7().to_string();
        self.id = Some(id.clone());
        let api_key = MockApiKey::builder().random()?.build();
        app.seed_user(self.clone()).await?;
        app.seed_api_key(api_key.clone(), &self).await?;
        Ok((self, api_key))
    }
}

impl MockUserBuilder {
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
        Ok(self.username(rand_helper::random_alphanumeric_string(1..=32)?.to_lowercase()))
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

    pub fn build(self) -> MockUser {
        let username = self.username.clone().unwrap_or("jmanderley".into());
        let email = self.email.clone().unwrap_or("jmanderley@unatco.gov".into());
        let password = self.password.clone().unwrap_or("knight_killer".into());

        MockUser {
            id: None,
            username,
            email,
            password,
        }
    }
}
