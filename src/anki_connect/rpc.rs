use serde::{Deserialize, Serialize};

/// Type for any RPC-able request.
pub trait AnkiRequest {
    type Response;

    const VERSION: u16;
    const ACTION: &'static str;
}

#[derive(Debug, Deserialize)]
pub struct CommandResponse<T> {
    pub result: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CommandRequest<T> {
    pub action: String,
    pub version: u16,
    pub params: T,
}

pub mod create_model {
    use crate::anki_connect::rpc::AnkiRequest;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateModelInput {
        pub model_name: String,
        pub in_order_fields: Vec<String>,
        pub css: String,
        pub is_cloze: bool,
        pub card_templates: Vec<CreateModelCardTemplateMessage>,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateModelCardTemplateMessage {
        #[serde(rename = "Name")]
        pub name: String,
        #[serde(rename = "Front")]
        pub front: String,
        #[serde(rename = "Back")]
        pub back: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateModelOutput {
        pub sortf: i64,
        pub did: Option<i64>,
        pub latex_pre: String,
        pub latex_post: String,
        pub r#mod: i64,
        pub usn: i64,
        #[serde(default)]
        pub vers: Vec<Value>,
        pub r#type: i64,
        pub css: String,
        pub name: String,
        pub flds: Vec<CreateModelFieldMessage>,
        pub tmpls: Vec<CreateModelTemplateMessage>,
        pub tags: Vec<Value>,
        pub id: i64,
        pub req: Value,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateModelFieldMessage {
        pub name: String,
        pub ord: i64,
        pub sticky: bool,
        pub rtl: bool,
        pub font: String,
        pub size: i64,
        #[serde(default)]
        pub media: Vec<Value>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateModelTemplateMessage {
        pub name: String,
        pub ord: i64,
        pub qfmt: String,
        pub afmt: String,
        pub did: Option<Value>,
        pub bqfmt: String,
        pub bafmt: String,
    }

    impl AnkiRequest for CreateModelInput {
        type Response = CreateModelOutput;

        const VERSION: u16 = 6;
        const ACTION: &'static str = "createModel";
    }
}

pub mod create_deck {
    use crate::anki_connect::rpc::AnkiRequest;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateDeckInput {
        pub deck: String,
    }

    impl AnkiRequest for CreateDeckInput {
        type Response = i64;

        const VERSION: u16 = 6;
        const ACTION: &'static str = "createDeck";
    }
}

pub mod add_note {
    use crate::anki_connect::rpc::AnkiRequest;
    use serde::Serialize;
    use serde_json::Value;
    use std::collections::HashMap;

    #[derive(Debug, Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct AddNoteInput {
        pub note: AddNoteNoteMessage,
    }

    #[derive(Debug, Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct AddNoteNoteMessage {
        pub deck_name: String,
        pub model_name: String,
        pub fields: HashMap<String, String>,
        pub tags: Vec<String>,
        pub audio: Vec<Value>,
        pub video: Vec<Value>,
        pub picture: Vec<Value>,
    }

    #[derive(Debug, Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct AddNoteOptionsMessage {
        pub allow_duplicate: bool,
        pub duplicate_scope: String,
        pub duplicate_scope_options: AddNoteDuplicateScopeOptionsMessage,
    }

    #[derive(Debug, Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct AddNoteDuplicateScopeOptionsMessage {
        pub deck_name: String,
        pub check_children: bool,
        pub check_all_models: bool,
    }

    impl AnkiRequest for AddNoteInput {
        type Response = i64;

        const VERSION: u16 = 6;
        const ACTION: &'static str = "addNote";
    }
}
