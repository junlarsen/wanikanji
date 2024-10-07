use anki_bridge::AnkiClient;
use thiserror::Error;

#[derive(Error)]
pub enum AnkiError {
    #[error("anki client error: {0}")]
    AnkiError(#[from] anki_bridge::Error),
}

pub struct AnkiConnectClient<'a> {
    pub client: AnkiClient<'a>,
}

impl AnkiConnectClient {
    pub fn from_env() -> AnkiConnectClient {
        let endpoint =
            std::env::var("ANKI_CONNECT_ENDPOINT").unwrap_or("http://localhost:8765".to_string());
        AnkiConnectClient {
            client: AnkiClient::new(&endpoint),
        }
    }
}
