use futures::future::BoxFuture;
use futures::FutureExt;
use reqwest::header::HeaderMap;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("http request error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("http request returned non-200 response: {0}")]
    QueryFailed(StatusCode),
}

#[derive(Debug)]
pub struct QueryClient {
    client: reqwest::Client,
}

impl QueryClient {
    pub fn from_token(token: Option<&str>) -> Self {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert(
                    "Wanikani-Revision",
                    "20170710"
                        .parse()
                        .expect("failed to parse string as header value"),
                );
                if let Some(token) = token {
                    headers.insert(
                        "Authorization",
                        format!("Bearer {}", token)
                            .parse()
                            .expect("failed to interpolate token into header value"),
                    );
                };
                headers
            })
            .build()
            .expect("failed to build reqwest client");
        Self { client }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get<'a, T: DeserializeOwned>(
        &'a self,
        url: &'a str,
    ) -> BoxFuture<'a, Result<T, QueryError>> {
        async move {
            let response = self.client.get(url).send().await?;
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
