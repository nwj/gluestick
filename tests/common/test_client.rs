#![allow(dead_code)]

use crate::common::mocks::mock_api_key::MockApiKey;
use crate::common::mocks::mock_pagination::{MockPaginationParams, MockPaginationResponse};
use crate::common::mocks::mock_paste::MockPaste;
use crate::common::mocks::mock_user::MockUser;
use crate::prelude::*;
use core::net::SocketAddr;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Response, Url};
use serde::Deserialize;

pub struct TestClient {
    base_url: Url,
    client: Client,
}

impl TestClient {
    pub fn new(address: SocketAddr, api_key: Option<&MockApiKey>) -> Result<Self> {
        let base_url = Url::parse(&format!("http://{address}/"))?;

        let mut headers = HeaderMap::new();
        if let Some(api_key) = api_key {
            headers.insert(
                "X-Gluestick-API-Key",
                HeaderValue::from_str(api_key.as_ref())?,
            );
        }

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .build()?;

        Ok(Self { base_url, client })
    }

    pub fn api_pastes(&self) -> ApiPastesEndpoint {
        ApiPastesEndpoint(self)
    }

    pub fn api_sessions(&self) -> ApiSessionsEndpoint {
        ApiSessionsEndpoint(self)
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

    pub fn pastes(&self) -> PastesEndpoint {
        PastesEndpoint(self)
    }

    pub fn settings(&self) -> SettingsEndpoint {
        SettingsEndpoint(self)
    }

    pub fn signup(&self) -> SignupEndpoint {
        SignupEndpoint(self)
    }

    pub fn username(&self, username: &String) -> UsernameEndpoint {
        UsernameEndpoint {
            client: self,
            username: username.to_string(),
        }
    }

    pub async fn get(&self) -> Result<Response> {
        Ok(self.client.get(self.base_url.clone()).send().await?)
    }

    pub async fn get_arbitrary(&self, endpoint: &str) -> Result<Response> {
        let url = self.base_url.join(endpoint)?;
        Ok(self.client.get(url).send().await?)
    }
}

pub struct ApiPastesEndpoint<'c>(&'c TestClient);

#[derive(Debug, Deserialize)]
pub struct ApiPastesIndexResponse {
    pub pastes: Vec<MockPaste>,
    pub pagination: MockPaginationResponse,
}

impl<'c> ApiPastesEndpoint<'c> {
    fn endpoint_str(&self) -> &str {
        "api/v1/pastes"
    }
    fn endpoint(&self) -> Result<Url> {
        Ok(self.0.base_url.join(self.endpoint_str())?)
    }

    fn endpoint_with_trailing_slash(&self) -> Result<Url> {
        Ok(self.0.base_url.join(&format!("{}/", self.endpoint_str()))?)
    }

    pub async fn get(&self, params: Option<MockPaginationParams>) -> Result<Response> {
        if let Some(params) = params {
            Ok(self
                .0
                .client
                .get(self.endpoint()?)
                .json(&params)
                .send()
                .await?)
        } else {
            Ok(self.0.client.get(self.endpoint()?).send().await?)
        }
    }

    pub async fn get_and_deserialize(
        &self,
        params: Option<MockPaginationParams>,
    ) -> Result<ApiPastesIndexResponse> {
        let response = self.get(params).await?;
        let response_data: ApiPastesIndexResponse = response.json().await?;
        Ok(response_data)
    }

    pub async fn post(&self, paste: &MockPaste) -> Result<Response> {
        Ok(self
            .0
            .client
            .post(self.endpoint()?)
            .json(&paste)
            .send()
            .await?)
    }

    pub async fn get_by_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self.endpoint_with_trailing_slash()?.join(&id)?;
        Ok(self.0.client.get(endpoint).send().await?)
    }

    pub async fn get_raw_by_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self
            .endpoint_with_trailing_slash()?
            .join(&format!("{id}/raw"))?;
        Ok(self.0.client.get(endpoint).send().await?)
    }

    pub async fn patch_by_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self.endpoint_with_trailing_slash()?.join(&id)?;
        Ok(self.0.client.patch(endpoint).json(&paste).send().await?)
    }

    pub async fn delete_by_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self.endpoint_with_trailing_slash()?.join(&id)?;
        Ok(self.0.client.delete(endpoint).send().await?)
    }
}

pub struct ApiSessionsEndpoint<'c>(&'c TestClient);

impl<'c> ApiSessionsEndpoint<'c> {
    fn endpoint_str(&self) -> &str {
        "api_sessions"
    }

    fn endpoint(&self) -> Result<Url> {
        Ok(self.0.base_url.join(self.endpoint_str())?)
    }

    fn endpoint_with_trailing_slash(&self) -> Result<Url> {
        Ok(self.0.base_url.join(&format!("{}/", self.endpoint_str()))?)
    }

    pub async fn post(&self) -> Result<Response> {
        Ok(self.0.client.post(self.endpoint()?).send().await?)
    }

    pub async fn delete_by_id(&self, api_key: &MockApiKey) -> Result<Response> {
        let endpoint = self.endpoint_with_trailing_slash()?.join(&api_key.id)?;
        Ok(self.0.client.delete(endpoint).send().await?)
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

    pub async fn post(&self, user: &MockUser) -> Result<Response> {
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

pub struct PastesEndpoint<'c>(&'c TestClient);

impl<'c> PastesEndpoint<'c> {
    fn endpoint_str(&self) -> &str {
        "pastes"
    }

    fn endpoint(&self) -> Result<Url> {
        Ok(self.0.base_url.join(self.endpoint_str())?)
    }

    fn endpoint_with_trailing_slash(&self) -> Result<Url> {
        Ok(self.0.base_url.join(&format!("{}/", self.endpoint_str()))?)
    }

    pub async fn get(&self, params: Option<MockPaginationParams>) -> Result<Response> {
        let mut url = self.endpoint()?;
        if let Some(params) = params {
            url = Url::parse_with_params(&url.to_string(), params.to_query_params())?;
        }
        Ok(self.0.client.get(url).send().await?)
    }

    pub async fn post(&self, paste: &MockPaste) -> Result<Response> {
        Ok(self
            .0
            .client
            .post(self.endpoint()?)
            .form(&[
                ("filename", &paste.filename),
                ("description", &paste.description),
                ("body", &paste.body),
                ("visibility", &paste.visibility),
            ])
            .send()
            .await?)
    }

    pub async fn get_new(&self) -> Result<Response> {
        let endpoint = self.endpoint_with_trailing_slash()?.join("new")?;
        Ok(self.0.client.get(endpoint).send().await?)
    }

    pub async fn delete_by_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self.endpoint_with_trailing_slash()?.join(&id)?;
        Ok(self.0.client.delete(endpoint).send().await?)
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

    pub async fn post(&self, invite_code: String, user: &MockUser) -> Result<Response> {
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

pub struct UsernameEndpoint<'c> {
    client: &'c TestClient,
    username: String,
}

impl<'c> UsernameEndpoint<'c> {
    fn endpoint(&self) -> Result<Url> {
        Ok(self.client.base_url.join(&self.username)?)
    }

    fn endpoint_with_trailing_slash(&self) -> Result<Url> {
        Ok(self.client.base_url.join(&format!("{}/", &self.username))?)
    }

    pub async fn get(&self, params: Option<MockPaginationParams>) -> Result<Response> {
        let mut url = self.endpoint()?;
        if let Some(params) = params {
            url = Url::parse_with_params(&url.to_string(), params.to_query_params())?;
        }
        Ok(self.client.client.get(url).send().await?)
    }

    pub async fn get_by_paste_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self.endpoint_with_trailing_slash()?.join(&id)?;
        Ok(self.client.client.get(endpoint).send().await?)
    }

    pub async fn get_raw_by_paste_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self
            .endpoint_with_trailing_slash()?
            .join(&format!("{id}/raw"))?;
        Ok(self.client.client.get(endpoint).send().await?)
    }

    pub async fn get_download_by_paste_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self
            .endpoint_with_trailing_slash()?
            .join(&format!("{id}/download"))?;
        Ok(self.client.client.get(endpoint).send().await?)
    }

    pub async fn get_edit_by_paste_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self
            .endpoint_with_trailing_slash()?
            .join(&format!("{id}/edit"))?;
        Ok(self.client.client.get(endpoint).send().await?)
    }

    pub async fn put_by_paste_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self.endpoint_with_trailing_slash()?.join(&id)?;
        Ok(self
            .client
            .client
            .put(endpoint)
            .form(&[
                ("filename", &paste.filename),
                ("description", &paste.description),
                ("body", &paste.body),
                ("visibility", &paste.visibility),
            ])
            .send()
            .await?)
    }

    pub async fn delete_by_paste_id(&self, paste: &MockPaste) -> Result<Response> {
        let id = paste.id.clone().unwrap_or_default();
        let endpoint = self.endpoint_with_trailing_slash()?.join(&id)?;
        Ok(self.client.client.delete(endpoint).send().await?)
    }
}
