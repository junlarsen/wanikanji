use crate::query::{
    ApiCollectionMessage, ApiObjectMessage, ApiSubjectMessage, QueryClient, QueryError,
};
use serde::{Deserialize, Serialize};

impl QueryClient {
    #[tracing::instrument(skip(self), err)]
    pub async fn list_vocabulary(&self) -> Result<Vec<ApiVocabularyMessage>, QueryError> {
        let mut next_url = Some("https://api.wanikani.com/v2/subjects?types=vocabulary".to_owned());
        let mut vocabulary = Vec::new();

        while let Some(url) = next_url {
            tracing::debug!("http query to get vocabulary by {}", &url);
            let response = self
                .get::<ApiCollectionMessage<ApiObjectMessage<ApiVocabularyMessage>>>(&url)
                .await?;
            let items = response.data.into_iter().map(|o| o.data);
            vocabulary.extend(items);
            next_url = response.pages.next_url;
        }
        Ok(vocabulary)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiVocabularyMessage {
    #[serde(flatten)]
    pub subject: ApiSubjectMessage,
    pub component_subject_ids: Vec<i32>,
    pub context_sentences: Vec<ApiVocabularyContextSentenceMessage>,
    pub parts_of_speech: Vec<String>,
    pub pronunciation_audios: Vec<ApiVocabularyPronunciationAudioMessage>,
    pub readings: Vec<ApiVocabularyReadingMessage>,
    pub reading_mnemonic: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiVocabularyContextSentenceMessage {
    pub en: String,
    pub ja: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiVocabularyPronunciationAudioMessage {
    pub url: String,
    pub content_type: String,
    pub metadata: ApiVocabularyPronunciationMetadataMessage,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiVocabularyPronunciationMetadataMessage {
    pub gender: String,
    pub source_id: i32,
    pub pronunciation: String,
    pub voice_actor_id: i32,
    pub voice_actor_name: String,
    pub voice_description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiVocabularyReadingMessage {
    pub accepted_answer: bool,
    pub primary: bool,
    pub reading: String,
}
