use crate::common::client::TestClient;
use crate::common::rand_helper;
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TestPaste {
    pub id: Option<String>,
    pub filename: String,
    pub description: String,
    pub body: String,
    pub visibility: String,
}

#[derive(Clone, Default)]
pub struct TestPasteBuilder {
    id: Option<String>,
    filename: Option<String>,
    description: Option<String>,
    body: Option<String>,
    visibility: Option<String>,
}

impl TestPaste {
    pub fn builder() -> TestPasteBuilder {
        TestPasteBuilder::new()
    }

    pub async fn persist(mut self, client: &TestClient) -> Result<Self> {
        let response = client.api_pastes().post(&self).await?;
        let id: Uuid = response.json().await?;
        self.id = Some(id.to_string());
        Ok(self)
    }
}

impl TestPasteBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        let _ = self.id.insert(id.into());
        self
    }

    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        let _ = self.filename.insert(filename.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        let _ = self.description.insert(description.into());
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        let _ = self.body.insert(body.into());
        self
    }

    #[allow(dead_code)]
    pub fn visibility(mut self, visibility: impl Into<String>) -> Self {
        let _ = self.visibility.insert(visibility.into());
        self
    }

    pub fn random_id(self) -> Self {
        self.id(Uuid::now_v7().to_string())
    }

    pub fn random_filename(self) -> Result<Self> {
        Ok(self.filename(rand_helper::random_filename(1..=256)?))
    }

    pub fn random_description(self) -> Result<Self> {
        Ok(self.description(rand_helper::random_string(1..=256)?))
    }

    pub fn random_body(self) -> Result<Self> {
        Ok(self.body(rand_helper::random_string(1..=1024)?))
    }

    // This does not set random id or visibility, since that's usually not what we want
    pub fn random(self) -> Result<Self> {
        Ok(self
            .random_filename()?
            .random_description()?
            .random_body()?)
    }

    pub fn build(self) -> TestPaste {
        let id = self.id.clone();
        let filename = self.filename.clone().unwrap_or("test.md".into());
        let description = self.description.clone().unwrap_or("A test paste".into());
        let body = self.body.clone().unwrap_or(
            "# Test Paste\n\nThis is a test of the emergency test paste system.\nBeep boop.".into(),
        );
        let visibility = self.visibility.clone().unwrap_or("public".into());

        TestPaste {
            id,
            filename,
            description,
            body,
            visibility,
        }
    }
}
