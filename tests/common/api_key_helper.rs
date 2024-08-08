use crate::common::app::TestApp;
use crate::common::rand_helper;
use crate::common::user_helper::TestUser;
use crate::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TestApiKey {
    pub id: String,
    pub name: String,
    pub unhashed_key: String,
}

#[derive(Clone, Default)]
pub struct TestApiKeyBuilder {
    id: Option<String>,
    name: Option<String>,
    unhashed_key: Option<String>,
}

impl TestApiKey {
    pub fn builder() -> TestApiKeyBuilder {
        TestApiKeyBuilder::new()
    }

    pub async fn seed(self, app: &TestApp, user: &TestUser) -> Result<Self> {
        app.seed_api_key(self.clone(), user).await?;
        Ok(self)
    }
}

impl From<TestApiKey> for String {
    fn from(value: TestApiKey) -> Self {
        value.unhashed_key
    }
}

impl AsRef<str> for TestApiKey {
    fn as_ref(&self) -> &str {
        &self.unhashed_key
    }
}

impl TestApiKeyBuilder {
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

    pub fn build(self) -> TestApiKey {
        let id = self.id.clone().unwrap_or(Uuid::now_v7().to_string());
        let name = self.name.clone().unwrap_or("API Key".into());
        let unhashed_key = self
            .unhashed_key
            .clone()
            .unwrap_or(rand_helper::random_api_key());

        TestApiKey {
            id,
            name,
            unhashed_key,
        }
    }
}
