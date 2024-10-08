use crate::anki_connect::client::{AnkiClient, AnkiError};
use crate::anki_connect::rpc::add_note::{AddNoteInput, AddNoteNoteMessage};
use crate::anki_connect::rpc::create_deck::CreateDeckInput;
use crate::anki_connect::rpc::create_model::{CreateModelCardTemplateMessage, CreateModelInput};
use crate::kanji::ApiKanjiMessage;
use crate::vocabulary::ApiVocabularyMessage;
use std::collections::HashMap;

impl AnkiClient<'_> {
    pub async fn create_kanji_model(&self, model_name: &str) -> Result<i64, AnkiError> {
        let request = CreateModelInput {
            model_name: model_name.to_owned(),
            css: include_str!("../res/anki.css").to_owned(),
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

    pub async fn create_vocabulary_model(&self, model_name: &str) -> Result<i64, AnkiError> {
        let request = CreateModelInput {
            model_name: model_name.to_owned(),
            css: include_str!("../res/anki.css").to_owned(),
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
                name: format!("Default type for '{}'", model_name),
                front: include_str!("../res/vocabulary-card-front.html").to_owned(),
                back: include_str!("../res/vocabulary-card-back.html").to_owned(),
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
