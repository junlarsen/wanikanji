use serde::Deserialize;

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

pub mod create_model {
    use crate::anki_connect::rpc::{AnkiRequest, CommandResponse};
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
        pub sortf: i32,
        pub did: i32,
        pub latex_pre: String,
        pub latex_post: String,
        pub r#mod: i32,
        pub usn: i32,
        pub vers: Vec<Value>,
        pub r#type: i32,
        pub css: String,
        pub name: String,
        pub flds: Vec<CreateModelFieldMessage>,
        pub tmpls: Vec<CreateModelTemplateMessage>,
        pub tags: Vec<Value>,
        pub id: i32,
        pub req: Value,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateModelFieldMessage {
        pub name: String,
        pub ord: i32,
        pub sticky: bool,
        pub rtl: bool,
        pub font: String,
        pub size: i32,
        pub media: Vec<Value>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CreateModelTemplateMessage {
        pub name: String,
        pub ord: i32,
        pub qfmt: String,
        pub afmt: String,
        pub did: Option<Value>,
        pub bqfmt: String,
        pub bafmt: String,
    }

    impl AnkiRequest for CreateModelInput {
        type Response = CommandResponse<CreateModelOutput>;

        const VERSION: u16 = 6;
        const ACTION: &'static str = "createModel";
    }
}
