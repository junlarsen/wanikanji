use crate::anki_connect::client::{AnkiClient, AnkiError};
use crate::io::FilesystemCache;
use crate::kanji::ApiKanjiMessage;
use crate::query::QueryClient;
use crate::vocabulary::ApiVocabularyMessage;
use clap::Parser;

pub mod anki;
pub mod anki_connect;
pub mod io;
pub mod kanji;
pub mod query;
pub mod vocabulary;

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

const DEFAULT_KANJI_DECK_NAME: &'static str = "Japanese Kanji";
const DEFAULT_VOCABULARY_DECK_NAME: &'static str = "Japanese Vocabulary";

#[derive(clap::Subcommand)]
pub enum Command {
    QueryKanji,
    QueryVocabulary,
    CreateKanjiDeck {
        #[clap(short, long, default_value = DEFAULT_KANJI_DECK_NAME)]
        name: String,
    },
    CreateVocabularyDeck {
        #[clap(short, long, default_value = DEFAULT_VOCABULARY_DECK_NAME)]
        name: String,
    },
    InstallKanji {
        #[clap(short, long, default_value = DEFAULT_KANJI_DECK_NAME)]
        deck_name: String,
        #[clap(short, long, default_value = DEFAULT_KANJI_DECK_NAME)]
        model_name: String,
    },
    InstallVocabulary {
        #[clap(short, long, default_value = DEFAULT_VOCABULARY_DECK_NAME)]
        deck_name: String,
        #[clap(short, long, default_value = DEFAULT_VOCABULARY_DECK_NAME)]
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
        Command::CreateKanjiDeck { name } => {
            anki_client.create_kanji_model(&name).await?;
            anki_client.create_deck(&name).await?;
        }
        Command::CreateVocabularyDeck { name } => {
            anki_client.create_vocabulary_model(&name).await?;
            anki_client.create_deck(&name).await?;
        }
        Command::InstallKanji {
            deck_name,
            model_name,
        } => {
            let kanji = cache.get::<Vec<ApiKanjiMessage>>("kanji").await?;
            match kanji {
                Some(kanji) => {
                    for kanji in kanji {
                        let input = kanji.into_anki_input(&model_name, &deck_name);
                        // SAFETY: This function has to perform a retry loop, because the Anki Connect API server tends to
                        // become overwhelmed with requests when it's fired off rapidly at the speed tokio+reqwest can perform.
                        fn is_connection_error(e: &AnkiError) -> bool {
                            matches!(e, AnkiError::HttpError(e) if e.is_connect())
                        }
                        again::retry_if(
                            || async {
                                match anki_client.send(input.clone()).await {
                                    Ok(_) => Ok(()),
                                    Err(AnkiError::ApiError(err))
                                        if err.contains(
                                            "cannot create note because it is a duplicate",
                                        ) =>
                                    {
                                        Ok(())
                                    }
                                    Err(e) => Err(e),
                                }
                            },
                            is_connection_error,
                        )
                        .await?;
                    }
                }
                None => {
                    tracing::error!("you must fetch kanji information before installing to deck")
                }
            }
        }
        Command::InstallVocabulary {
            deck_name,
            model_name,
        } => {
            let vocabulary = cache.get::<Vec<ApiVocabularyMessage>>("vocabulary").await?;
            match vocabulary {
                Some(vocabulary) => {
                    for vocabulary in vocabulary {
                        let input = vocabulary.into_anki_input(&model_name, &deck_name);
                        // SAFETY: This function has to perform a retry loop, because the Anki Connect API server tends to
                        // become overwhelmed with requests when it's fired off rapidly at the speed tokio+reqwest can perform.
                        fn is_connection_error(e: &AnkiError) -> bool {
                            matches!(e, AnkiError::HttpError(e) if e.is_connect())
                        }
                        again::retry_if(
                            || async {
                                match anki_client.send(input.clone()).await {
                                    Ok(_) => Ok(()),
                                    Err(AnkiError::ApiError(err))
                                        if err.contains(
                                            "cannot create note because it is a duplicate",
                                        ) =>
                                    {
                                        Ok(())
                                    }
                                    Err(e) => Err(e),
                                }
                            },
                            is_connection_error,
                        )
                        .await?;
                    }
                }
                None => {
                    tracing::error!(
                        "you must fetch vocabulary information before installing to deck"
                    )
                }
            }
        }
    }
    Ok(())
}
