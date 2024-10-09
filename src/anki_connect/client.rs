use crate::anki_connect::rpc::{AnkiRequest, CommandRequest, CommandResponse};
use reqwest::Client;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnkiError {
    #[error("anki http request error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("anki data serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("anki endpoint returned nothing")]
    EmptyResponse,
    #[error("anki endpoint returned error: {0}")]
    ApiError(String),
    #[error("io error: {0}")]
    Io(#[from] tokio::io::Error),
}

pub struct AnkiClient<'a> {
    client: Client,
    endpoint: &'a str,
}

impl Default for AnkiClient<'_> {
    fn default() -> Self {
        Self::from_endpoint("http://localhost:8765")
    }
}

impl<'a> AnkiClient<'a> {
    pub fn from_endpoint(endpoint: &'a str) -> Self {
        Self {
            client: Client::builder()
                .tcp_keepalive(Some(Duration::from_secs(60)))
                .build()
                .unwrap(),
            endpoint,
        }
    }

    /// Send a HTTP request to the Anki Connect API.
    pub async fn send<T>(&self, request: T) -> Result<T::Response, AnkiError>
    where
        T: AnkiRequest,
        T: serde::Serialize,
        T::Response: for<'de> serde::Deserialize<'de>,
    {
        let request = CommandRequest {
            action: T::ACTION.to_owned(),
            version: T::VERSION,
            params: request,
        };
        let response = self
            .client
            .post(self.endpoint)
            .json(&request)
            .send()
            .await?;
        let response = response.json::<CommandResponse<T::Response>>().await?;

        match (response.result, response.error) {
            (Some(result), _) => Ok(result),
            (None, Some(error)) => Err(AnkiError::ApiError(error)),
            (None, None) => Err(AnkiError::EmptyResponse),
        }
    }
}
