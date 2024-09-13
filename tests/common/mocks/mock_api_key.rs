use crate::common::mocks::mock_user::MockUser;
use crate::common::rand_helper;
use crate::common::test_app::TestApp;
use crate::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct MockApiKey {
    pub id: String,
    pub name: String,
    pub unhashed_key: String,
}

#[derive(Clone, Default)]
pub struct MockApiKeyBuilder {
    id: Option<String>,
    name: Option<String>,
    unhashed_key: Option<String>,
}

impl MockApiKey {
    pub fn builder() -> MockApiKeyBuilder {
        MockApiKeyBuilder::new()
    }

    pub async fn seed(self, app: &TestApp, user: &MockUser) -> Result<Self> {
        app.seed_api_key(self.clone(), user).await?;
        Ok(self)
    }
}

impl From<MockApiKey> for String {
    fn from(value: MockApiKey) -> Self {
        value.unhashed_key
    }
}

impl AsRef<str> for MockApiKey {
    fn as_ref(&self) -> &str {
        &self.unhashed_key
    }
}

impl MockApiKeyBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        let _ = self.name.insert(name.into());
        self
    }

    pub fn random_name(self) -> Result<Self> {
        Ok(self.name(rand_helper::random_string(1..=256)?))
    }

    pub fn random(self) -> Result<Self> {
        Ok(self.random_name()?)
    }

    pub fn build(self) -> MockApiKey {
        let id = self.id.clone().unwrap_or(Uuid::now_v7().to_string());
        let name = self.name.clone().unwrap_or("API Key".into());
        let unhashed_key = self
            .unhashed_key
            .clone()
            .unwrap_or(rand_helper::random_api_key());

        MockApiKey {
            id,
            name,
            unhashed_key,
        }
    }
}
