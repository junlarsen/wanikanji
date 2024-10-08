use crate::anki::into_anki_input;
use crate::anki_connect::client::{AnkiClient, AnkiError};
use crate::io::FilesystemCache;
use crate::kanji::ApiKanjiMessage;
use crate::query::QueryClient;
use clap::Parser;

mod anki;
pub mod anki_connect;
mod io;
pub mod kanji;
pub mod query;
mod vocabulary;

#[derive(clap::Parser)]
pub struct Options {
    #[clap(subcommand)]
    pub command: Command,
    #[clap(long, default_value = ".cache")]
    pub cache_dir: String,
    #[clap(long)]
    pub api_token: Option<String>,
    #[clap(long, default_value = "http://localhost:8765")]
    pub anki_endpoint: String,
}

#[derive(clap::Subcommand)]
pub enum Command {
    QueryKanji,
    QueryVocabulary,
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
    let cache = FilesystemCache::new(&args.cache_dir).await?;
    let wanikani_client = QueryClient::from_token(args.api_token.as_deref());
    let anki_client = AnkiClient::from_endpoint(&args.anki_endpoint);

    match args.command {
        Command::QueryKanji => {
            let kanji = wanikani_client.list_kanji().await?;
            cache.insert("kanji", &kanji).await?;
        }
        Command::QueryVocabulary => {
            let vocabulary = wanikani_client.list_vocabulary().await?;
            cache.insert("vocabulary", &vocabulary).await?;
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
            let kanji = cache.get::<Vec<ApiKanjiMessage>>("kanji").await?;
            match kanji {
                Some(kanji) => {
                    for kanji in kanji {
                        let input = into_anki_input(kanji, &model_name, &deck_name);
                        // SAFETY: This function has to perform a retry loop, because the Anki Connect API server tends to
                        // become overwhelmed with requests when it's fired off rapidly at the speed tokio+reqwest can perform.
                        fn is_connection_error(e: &AnkiError) -> bool {
                            matches!(e, AnkiError::HttpError(e) if e.is_connect())
                        }
                        again::retry_if(|| anki_client.send(input.clone()), is_connection_error)
                            .await?;
                    }
                }
                None => {
                    tracing::error!("you must fetch kanji information before installing to deck")
                }
            }
        }
    }
    Ok(())
}
