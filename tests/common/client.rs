#![allow(dead_code)]

use crate::common::user_helper::TestUser;
use crate::prelude::*;
use core::net::SocketAddr;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Response, Url};

pub struct TestClient {
    base_url: Url,
    client: Client,
}

impl TestClient {
    pub fn new(address: SocketAddr, api_key: Option<&str>) -> Result<Self> {
        let base_url = Url::parse(&format!("http://{address}/"))?;

        let mut headers = HeaderMap::new();
        if let Some(api_key) = api_key {
            headers.insert("X-Gluestick-API-Key", HeaderValue::from_str(api_key)?);
        }

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .build()?;

        Ok(Self { base_url, client })
    }

    pub fn health(&self) -> HealthEndpoint {
        HealthEndpoint(self)
    }

    pub fn login(&self) -> LoginEndpoint {
        LoginEndpoint(self)
    }

    pub fn logout(&self) -> LogoutEndpoint {
        LogoutEndpoint(self)
    }

    pub fn settings(&self) -> SettingsEndpoint {
        SettingsEndpoint(self)
    }

    pub fn signup(&self) -> SignupEndpoint {
        SignupEndpoint(self)
    }
}

pub struct HealthEndpoint<'c>(&'c TestClient);

impl<'c> HealthEndpoint<'c> {
    fn endpoint(&self) -> Result<Url> {
        Ok(self.0.base_url.join("health")?)
    }

    pub async fn get(&self) -> Result<Response> {
        Ok(self.0.client.get(self.endpoint()?).send().await?)
    }
}

pub struct LoginEndpoint<'c>(&'c TestClient);

impl<'c> LoginEndpoint<'c> {
    fn endpoint(&self) -> Result<Url> {
        Ok(self.0.base_url.join("login")?)
    }

    pub async fn get(&self) -> Result<Response> {
        Ok(self.0.client.get(self.endpoint()?).send().await?)
    }

    pub async fn post(&self, user: &TestUser) -> Result<Response> {
        Ok(self
            .0
            .client
            .post(self.endpoint()?)
            .form(&[("email", &user.email), ("password", &user.password)])
            .send()
            .await?)
    }
}

pub struct LogoutEndpoint<'c>(&'c TestClient);

impl<'c> LogoutEndpoint<'c> {
    fn endpoint(&self) -> Result<Url> {
        Ok(self.0.base_url.join("logout")?)
    }

    pub async fn delete(&self) -> Result<Response> {
        Ok(self.0.client.delete(self.endpoint()?).send().await?)
    }
}

pub struct SettingsEndpoint<'c>(&'c TestClient);

impl<'c> SettingsEndpoint<'c> {
    fn endpoint(&self) -> Result<Url> {
        Ok(self.0.base_url.join("settings")?)
    }

    pub async fn get(&self) -> Result<Response> {
        Ok(self.0.client.get(self.endpoint()?).send().await?)
    }
}

pub struct SignupEndpoint<'c>(&'c TestClient);

impl<'c> SignupEndpoint<'c> {
    fn endpoint(&self) -> Result<Url> {
        Ok(self.0.base_url.join("signup")?)
    }

    pub async fn get(&self) -> Result<Response> {
        Ok(self.0.client.get(self.endpoint()?).send().await?)
    }

    pub async fn post(&self, invite_code: String, user: &TestUser) -> Result<Response> {
        Ok(self
            .0
            .client
            .post(self.endpoint()?)
            .form(&[
                ("username", &user.username),
                ("email", &user.email),
                ("password", &user.password),
                ("invite_code", &invite_code),
            ])
            .send()
            .await?)
    }
}
