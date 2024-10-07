use crate::anki::{AnkiConnectClient, AnkiError};
use crate::query::{
    ApiCollectionMessage, ApiObjectMessage, ApiSubjectMessage, QueryClient, QueryError,
};
use serde::{Deserialize, Serialize};

impl QueryClient {
    #[tracing::instrument(skip(self), err)]
    pub async fn list_kanji(&self) -> Result<Vec<ApiKanjiMessage>, QueryError> {
        let mut next_url = Some("https://api.wanikani.com/v2/subjects?types=kanji".to_owned());
        let mut kanji = Vec::new();

        while let Some(url) = next_url {
            tracing::debug!("http query to get kanji by {}", &url);
            let response = self
                .get::<ApiCollectionMessage<ApiObjectMessage<ApiKanjiMessage>>>(&url)
                .await?;
            let items = response.data.into_iter().map(|o| o.data);
            kanji.extend(items);
            next_url = response.pages.next_url;
        }
        Ok(kanji)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiKanjiMessage {
    #[serde(flatten)]
    pub subject: ApiSubjectMessage,
    /// The list of vocabulary subjects that use this kanji.
    pub amalgamation_subject_ids: Vec<i32>,
    /// The list of radicals that this kanji is composed of.
    pub component_subject_ids: Vec<i32>,
    pub meaning_hint: Option<String>,
    pub reading_hint: Option<String>,
    pub reading_mnemonic: String,
    pub readings: Vec<ApiKanjiReadingMessage>,
    pub visually_similar_subject_ids: Vec<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiKanjiReadingMessage {
    pub reading: String,
    pub primary: bool,
    pub accepted_answer: bool,
    pub r#type: String,
}

impl AnkiConnectClient {
    #[tracing::instrument(skip(self), err)]
    pub async fn create_model(&self) -> Result<(), AnkiError> {
        todo!()
    }
}
