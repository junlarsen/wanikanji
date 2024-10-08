use crate::anki::into_anki_input;
use crate::anki_connect::client::{AnkiClient, AnkiError};
use crate::kanji::ApiKanjiMessage;
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
    AddCardModel {
        #[clap(short, long, default_value = "Japanese Kanji")]
        name: String,
    },
    AddCardDeck {
        #[clap(short, long, default_value = "Japanese Kanji")]
        name: String,
    },
    InstallKanji {
        #[clap(short, long, default_value = "Japanese Kanji")]
        deck_name: String,
        #[clap(short, long, default_value = "Japanese Kanji")]
        model_name: String,
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
        Command::AddCardModel { name } => {
            let client = AnkiClient::default();
            client.create_model(&name).await?;
        }
        Command::AddCardDeck { name } => {
            let client = AnkiClient::default();
            client.create_deck(&name).await?;
        }
        Command::InstallKanji {
            deck_name,
            model_name,
        } => {
            if tokio::fs::metadata(".cache").await.is_err() {
                tracing::warn!(
                    "no .cache directory found, please install kanji from wanikani first"
                );
                return Ok(());
            }
            let kanji = tokio::fs::read(".cache/kanji.json").await?;
            let kanji: Vec<ApiKanjiMessage> = serde_json::from_slice(&kanji)?;
            let client = AnkiClient::default();
            for kanji in kanji {
                let input = into_anki_input(kanji, &model_name, &deck_name);
                // SAFETY: This function has to perform a retry loop, because the Anki Connect API server tends to
                // become overwhelmed with requests when it's fired off rapidly at the speed tokio+reqwest can perform.
                fn is_connection_error(e: &AnkiError) -> bool {
                    matches!(e, AnkiError::HttpError(e) if e.is_connect())
                }
                again::retry_if(|| client.send(input.clone()), is_connection_error).await?;
            }
        }
    }
    Ok(())
}
