use crate::common::rand_helper;
use crate::common::TestApp;
use crate::prelude::*;
use reqwest::{Client, Response};

#[derive(Debug)]
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

    pub async fn signup(
        &self,
        app: &TestApp,
        client: &Client,
        invite_code: String,
    ) -> Result<Response> {
        let response = client
            .post(format!("http://{}/signup", app.address))
            .form(&[
                ("username", &self.username),
                ("email", &self.email),
                ("password", &self.password),
                ("invite_code", &invite_code),
            ])
            .send()
            .await?;
        Ok(response)
    }

    pub async fn login(&self, app: &TestApp, client: &Client) -> Result<Response> {
        let response = client
            .post(format!("http://{}/login", app.address))
            .form(&[("email", &self.email), ("password", &self.password)])
            .send()
            .await?;
        Ok(response)
    }

    pub async fn logout(&self, app: &TestApp, client: &Client) -> Result<Response> {
        let response = client
            .delete(format!("http://{}/logout", app.address))
            .send()
            .await?;
        Ok(response)
    }

    pub async fn settings(&self, app: &TestApp, client: &Client) -> Result<Response> {
        let response = client
            .get(format!("http://{}/settings", app.address))
            .send()
            .await?;
        Ok(response)
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
        Ok(self.email(rand_helper::random_email(6..=35)?))
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
