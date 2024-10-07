use futures::future::BoxFuture;
use futures::FutureExt;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("failed to read api token")]
    MissingTokenError,
    #[error("http request error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("http request returned non-200 response: {0}")]
    QueryFailed(StatusCode),
}

#[derive(Debug)]
pub struct QueryClient {
    client: reqwest::Client,
    api_token: String,
}

impl QueryClient {
    /// Construct a new query client using the API_TOKEN environment variable as the WaniKani API
    /// token.
    pub fn from_env() -> Result<Self, QueryError> {
        let api_token = std::env::var("API_TOKEN").map_err(|_| QueryError::MissingTokenError)?;
        let client = reqwest::Client::new();
        Ok(Self { client, api_token })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get<'a, T: DeserializeOwned>(
        &'a self,
        url: &'a str,
    ) -> BoxFuture<'a, Result<T, QueryError>> {
        async move {
            let response = self
                .client
                .get(url)
                .header("Authorization", format!("Bearer {}", self.api_token))
                .header("Wanikani-Revision", "20170710")
                .send()
                .await?;
            if !response.status().is_success() {
                tracing::error!("request failed: {:?}", response.status());
                if response.status() == StatusCode::TOO_MANY_REQUESTS {
                    tracing::warn!(
                        "rate limit reached for api token, retrying request in 60 seconds"
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    return self.get(url).await;
                }
                return Err(QueryError::QueryFailed(response.status()));
            }
            let response = response.json::<T>().await?;
            Ok(response)
        }
        .boxed()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiObjectMessage<T> {
    pub object: String,
    pub url: String,
    pub data_updated_at: Option<String>,
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiCollectionMessage<T> {
    pub object: String,
    pub url: String,
    pub data_updated_at: Option<String>,
    pub data: Vec<T>,
    pub total_count: i32,
    pub pages: ApiPaginationMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiPaginationMessage {
    pub per_page: i32,
    pub next_url: Option<String>,
    pub previous_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiSubjectMessage {
    pub auxiliary_meanings: Vec<ApiAuxiliaryMeaningMessage>,
    /// The character(s) that make up the subject's item. This value can be null only if the subject
    /// is a radical item.
    pub characters: Option<String>,
    pub created_at: String,
    pub document_url: String,
    pub hidden_at: Option<String>,
    pub lesson_position: i32,
    pub level: i32,
    pub meaning_mnemonic: String,
    pub meanings: Vec<ApiMeaningMessage>,
    pub slug: String,
    pub spaced_repetition_system_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiMeaningMessage {
    pub meaning: String,
    pub primary: bool,
    pub accepted_answer: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiAuxiliaryMeaningMessage {
    pub meaning: String,
    pub r#type: String,
}
