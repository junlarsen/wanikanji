use crate::anki_connect::rpc::AnkiRequest;
use reqwest::Client;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnkiError {
    #[error("anki http request error: {0}")]
    AnkiError(#[from] reqwest::Error),
    #[error("anki data serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
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
            client: Client::new(),
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
        let request = serde_json::to_string(&request)?;
        let response = self
            .client
            .post(self.endpoint)
            .json(&request)
            .send()
            .await?;
        let response = response.json::<T::Response>().await?;
        Ok(response)
    }
}
