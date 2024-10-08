use crate::anki_connect::client::AnkiClient;
use crate::anki_connect::rpc::create_model::{CreateModelCardTemplateMessage, CreateModelInput};
use crate::query::QueryClient;
use clap::Parser;

mod anki;
pub mod anki_connect;
pub mod kanji;
pub mod query;

#[derive(clap::Parser)]
pub struct Options {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    QueryKanji,
    CreateCardType {
        #[clap(short, long, default_value = "Japanese Kanji")]
        deck_name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Options::parse();
    match args.command {
        Command::QueryKanji => {
            let client = QueryClient::from_env()?;
            let kanji = client.list_kanji().await?;
            if tokio::fs::metadata(".cache").await.is_err() {
                tokio::fs::create_dir(".cache").await?;
            }
            tokio::fs::write(".cache/kanji.json", serde_json::to_string_pretty(&kanji)?).await?;
        }
        Command::CreateCardType { deck_name } => {
            let client = AnkiClient::default();
            let request = CreateModelInput {
                model_name: "Kanji".to_owned(),
                css: include_str!("../res/kanji-card.css").to_owned(),
                is_cloze: false,
                in_order_fields: vec![
                    "kanji".to_owned(),
                    "meanings".to_owned(),
                    "meaning-mnemonic".to_owned(),
                    "readings".to_owned(),
                    "reading-mnemonic".to_owned(),
                    "reference-url".to_owned(),
                ],
                card_templates: vec![CreateModelCardTemplateMessage {
                    name: "Default card type".to_owned(),
                    front: include_str!("../res/kanji-card-front.html").to_owned(),
                    back: include_str!("../res/kanji-card-back.html").to_owned(),
                }],
            };
            client.send(request).await?;
        }
    }

    Ok(())
}
