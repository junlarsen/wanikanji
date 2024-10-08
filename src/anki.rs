use crate::anki_connect::client::{AnkiClient, AnkiError};
use crate::anki_connect::rpc::add_note::{AddNoteInput, AddNoteNoteMessage};
use crate::anki_connect::rpc::create_deck::CreateDeckInput;
use crate::anki_connect::rpc::create_model::{CreateModelCardTemplateMessage, CreateModelInput};
use crate::anki_connect::rpc::update_model_styling::{
    UpdateModelModelMessage, UpdateModelStylingInput,
};
use crate::anki_connect::rpc::update_model_templates::{
    UpdateModelCardTemplateMessage, UpdateModelContentMessage, UpdateModelTemplatesInput,
};
use crate::kanji::ApiKanjiMessage;
use crate::vocabulary::ApiVocabularyMessage;
use crate::ConfigurationDeckOptions;
use std::collections::HashMap;

impl AnkiClient<'_> {
    pub async fn update_model_styling(&self, model_name: &str) -> Result<(), AnkiError> {
        let request = UpdateModelStylingInput {
            model: UpdateModelModelMessage {
                name: model_name.to_owned(),
                css: tokio::fs::read_to_string("res/anki.css").await?,
            },
        };
        match self.send(request).await {
            Ok(_) => Ok(()),
            Err(AnkiError::EmptyResponse) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn update_model_templates(
        &self,
        model: &ConfigurationDeckOptions,
    ) -> Result<(), AnkiError> {
        let request = UpdateModelTemplatesInput {
            model: UpdateModelContentMessage {
                name: model.model_name.to_owned(),
                templates: HashMap::from([(
                    model.model_template_name.to_owned(),
                    UpdateModelCardTemplateMessage {
                        front: tokio::fs::read_to_string(model.model_template_front.clone())
                            .await?,
                        back: tokio::fs::read_to_string(model.model_template_back.clone()).await?,
                    },
                )]),
            },
        };
        match self.send(request).await {
            Ok(_) => Ok(()),
            Err(AnkiError::EmptyResponse) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn create_kanji_model(
        &self,
        model_name: &str,
        model_template_name: &str,
    ) -> Result<i64, AnkiError> {
        let request = CreateModelInput {
            model_name: model_name.to_owned(),
            css: tokio::fs::read_to_string("res/anki.css").await?,
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
                name: model_template_name.to_owned(),
                front: tokio::fs::read_to_string("res/kanji-card-front.html").await?,
                back: tokio::fs::read_to_string("res/kanji-card-back.html").await?,
            }],
        };
        Ok(self.send(request).await?.id)
    }

    pub async fn create_vocabulary_model(
        &self,
        model_name: &str,
        model_template_name: &str,
    ) -> Result<i64, AnkiError> {
        let request = CreateModelInput {
            model_name: model_name.to_owned(),
            css: tokio::fs::read_to_string("res/anki.css").await?,
            is_cloze: false,
            in_order_fields: vec![
                "vocabulary".to_owned(),
                "primary-meaning".to_owned(),
                "primary-meaning-mnemonic".to_owned(),
                "secondary-meanings".to_owned(),
                "primary-reading".to_owned(),
                "primary-reading-mnemonic".to_owned(),
                "secondary-readings".to_owned(),
                "context-sentence-1-en".to_owned(),
                "context-sentence-1-ja".to_owned(),
                "context-sentence-2-en".to_owned(),
                "context-sentence-2-ja".to_owned(),
                "context-sentence-3-en".to_owned(),
                "context-sentence-3-ja".to_owned(),
                "reference-url".to_owned(),
            ],
            card_templates: vec![CreateModelCardTemplateMessage {
                name: model_template_name.to_owned(),
                front: tokio::fs::read_to_string("res/vocabulary-card-front.html").await?,
                back: tokio::fs::read_to_string("res/vocabulary-card-back.html").await?,
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

impl ApiKanjiMessage {
    pub fn into_anki_input(self, model_name: &str, deck_name: &str) -> AddNoteInput {
        let attributes = {
            let mut attr = HashMap::new();
            // SAFETY: All kanji have a characters field. Only radical items may not have a characters field.
            attr.insert(
                "kanji".to_owned(),
                self.subject
                    .characters
                    .expect("kanji must have a characters field")
                    .to_owned(),
            );

            let primary_meaning = self
                .subject
                .meanings
                .iter()
                .find(|m| m.primary)
                .expect("kanji must have a meaning marked primary");
            attr.insert(
                "primary-meaning".to_owned(),
                primary_meaning.meaning.clone(),
            );

            attr.insert(
                "primary-meaning-mnemonic".to_owned(),
                self.subject.meaning_mnemonic.clone(),
            );
            attr.insert(
                "primary-reading-mnemonic".to_owned(),
                self.reading_mnemonic.clone(),
            );
            attr.insert(
                "reference-url".to_owned(),
                self.subject.document_url.clone(),
            );

            let alternative_meanings = self
                .subject
                .meanings
                .iter()
                .filter(|m| !m.primary)
                .map(|m| m.meaning.clone())
                .collect::<Vec<_>>()
                .join(", ");
            attr.insert("secondary-meanings".to_owned(), alternative_meanings);

            let readings = self
                .readings
                .iter()
                .map(|r| r.reading.clone())
                .collect::<Vec<_>>()
                .join(", ");
            attr.insert("readings".to_owned(), readings);

            attr
        };
        AddNoteInput {
            note: AddNoteNoteMessage {
                deck_name: deck_name.to_owned(),
                model_name: model_name.to_owned(),
                tags: vec!["WaniKani Kanji".to_owned()],
                audio: vec![],
                picture: vec![],
                video: vec![],
                fields: attributes,
            },
        }
    }
}

impl ApiVocabularyMessage {
    pub fn into_anki_input(self, model_name: &str, deck_name: &str) -> AddNoteInput {
        let attributes = {
            let mut attr = HashMap::new();
            // SAFETY: All kanji have a characters field. Only radical items may not have a characters field.
            attr.insert(
                "vocabulary".to_owned(),
                self.subject
                    .characters
                    .expect("vocabulary must have a characters field")
                    .to_owned(),
            );

            let primary_meaning = self
                .subject
                .meanings
                .iter()
                .find(|m| m.primary)
                .expect("vocabulary must have a meaning marked primary");
            attr.insert(
                "primary-meaning".to_owned(),
                primary_meaning.meaning.clone(),
            );

            attr.insert(
                "primary-meaning-mnemonic".to_owned(),
                self.subject.meaning_mnemonic.clone(),
            );
            attr.insert(
                "primary-reading-mnemonic".to_owned(),
                self.reading_mnemonic.clone(),
            );
            attr.insert(
                "reference-url".to_owned(),
                self.subject.document_url.clone(),
            );

            let alternative_meanings = self
                .subject
                .meanings
                .iter()
                .filter(|m| !m.primary)
                .map(|m| m.meaning.clone())
                .collect::<Vec<_>>()
                .join(", ");
            attr.insert("secondary-meanings".to_owned(), alternative_meanings);

            let primary_reading = self
                .readings
                .iter()
                .find(|r| r.primary)
                .expect("vocabulary must have a reading marked primary");
            attr.insert(
                "primary-reading".to_owned(),
                primary_reading.reading.clone(),
            );

            let secondary_readings = self
                .readings
                .iter()
                .filter(|r| !r.primary)
                .map(|r| r.reading.clone())
                .collect::<Vec<_>>()
                .join(", ");
            attr.insert("readings".to_owned(), secondary_readings);

            let relevant_context_sentences = self.context_sentences.iter().take(3);
            for (i, sentence) in relevant_context_sentences.enumerate() {
                attr.insert(format!("context-sentence-{}-ja", i), sentence.ja.clone());
                attr.insert(format!("context-sentence-{}-en", i), sentence.en.clone());
            }

            attr
        };
        AddNoteInput {
            note: AddNoteNoteMessage {
                deck_name: deck_name.to_owned(),
                model_name: model_name.to_owned(),
                tags: vec!["WaniKani Vocabulary".to_owned()],
                audio: vec![],
                picture: vec![],
                video: vec![],
                fields: attributes,
            },
        }
    }
}
