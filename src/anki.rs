use crate::anki_connect::client::{AnkiClient, AnkiError};
use crate::anki_connect::rpc::create_deck::CreateDeckInput;
use crate::anki_connect::rpc::create_model::{CreateModelCardTemplateMessage, CreateModelInput};

impl AnkiClient<'_> {
    pub async fn create_model(&self, model_name: &str) -> Result<i64, AnkiError> {
        let request = CreateModelInput {
            model_name: model_name.to_owned(),
            css: include_str!("../res/kanji-card.css").to_owned(),
            is_cloze: false,
            in_order_fields: vec![
                "kanji".to_owned(),
                "primary-meaning".to_owned(),
                "primary-meaning-mnemonic".to_owned(),
                "secondary-meanings".to_owned(),
                "primary-reading-mnemonic".to_owned(),
                "readings".to_owned(),
                "reference-url".to_owned(),
            ],
            card_templates: vec![CreateModelCardTemplateMessage {
                name: format!("Default type for '{}'", model_name),
                front: include_str!("../res/kanji-card-front.html").to_owned(),
                back: include_str!("../res/kanji-card-back.html").to_owned(),
            }],
        };
        Ok(self.send(request).await?.id)
    }

    pub async fn create_deck(&self, deck_name: &str) -> Result<i64, AnkiError> {
        let request = CreateDeckInput {
            deck: deck_name.to_owned(),
        };
        self.send(request).await
    }
}
