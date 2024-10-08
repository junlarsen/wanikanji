use anki_bridge::model_actions::model_names::ModelNamesRequest;
use anki_bridge::model_actions::model_names_and_ids::ModelNamesAndIdsRequest;
use anki_bridge::{AnkiClient, AnkiRequestable};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnkiError {
    #[error("anki client error: {0}")]
    AnkiError(#[from] anki_bridge::Error),
}

pub struct AnkiConnectClient<'a> {
    pub client: AnkiClient<'a>,
}

impl<'a> AnkiConnectClient<'a> {
    /// Create a new Anki Connect client that connects to the default endpoint.
    pub fn new() -> AnkiConnectClient<'a> {
        AnkiConnectClient {
            client: AnkiClient::default(),
        }
    }

    /// Find a model type by name.
    pub async fn find_model(&self, name: &str) -> Result<Option<usize>, AnkiError> {
        let models = self.client.request(ModelNamesAndIdsRequest {}).await?;
        Ok(models.get(name).map(|id| Some(*id)).unwrap_or(None))
    }
}
