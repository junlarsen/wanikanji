use crate::anki_connect::client::{AnkiClient, AnkiError};
use crate::io::FilesystemCache;
use crate::kanji::ApiKanjiMessage;
use crate::query::QueryClient;
use crate::vocabulary::ApiVocabularyMessage;
use clap::Parser;
use config::Config;
use serde::Deserialize;

pub mod anki;
pub mod anki_connect;
pub mod io;
pub mod kanji;
pub mod query;
pub mod vocabulary;

#[derive(clap::Parser)]
#[clap(author, version, about = "Export your WaniKani data into Anki decks", long_about = None)]
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
    #[clap(about = "Download all kanji data from wanikani")]
    QueryKanji,
    #[clap(about = "Download all vocabulary data from wanikani")]
    QueryVocabulary,
    #[clap(about = "Create Anki deck and Anki card type for Kanji")]
    CreateKanjiDeck,
    #[clap(about = "Create Anki deck and Anki card type for Vocabulary")]
    CreateVocabularyDeck,
    #[clap(about = "Install previously downloaded Kanji data into Anki deck")]
    InstallKanji,
    #[clap(about = "Install previously downloaded Vocabulary data into Anki deck")]
    InstallVocabulary,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub kanji: ConfigurationDeckOptions,
    pub vocabulary: ConfigurationDeckOptions,
}

#[derive(Debug, Deserialize)]
pub struct ConfigurationDeckOptions {
    pub deck_name: String,
    pub model_name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let settings = Config::builder()
        .add_source(config::File::with_name("wanikanji"))
        .build()?;
    let configuration = settings
        .try_deserialize::<Configuration>()
        .expect("failed to deserialize configuration, does wanikanji.toml exist?");
    tracing::debug!("running wanikanji with configuration {:?}", &configuration);

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
        Command::CreateKanjiDeck => {
            anki_client
                .create_kanji_model(&configuration.kanji.model_name)
                .await?;
            anki_client
                .create_deck(&configuration.kanji.deck_name)
                .await?;
        }
        Command::CreateVocabularyDeck => {
            anki_client
                .create_vocabulary_model(&configuration.vocabulary.model_name)
                .await?;
            anki_client
                .create_deck(&configuration.vocabulary.deck_name)
                .await?;
        }
        Command::InstallKanji => {
            let kanji = cache.get::<Vec<ApiKanjiMessage>>("kanji").await?;
            match kanji {
                Some(kanji) => {
                    for kanji in kanji {
                        let input = kanji.into_anki_input(
                            &configuration.kanji.model_name,
                            &configuration.kanji.deck_name,
                        );
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
        Command::InstallVocabulary => {
            let vocabulary = cache.get::<Vec<ApiVocabularyMessage>>("vocabulary").await?;
            match vocabulary {
                Some(vocabulary) => {
                    for vocabulary in vocabulary {
                        let input = vocabulary.into_anki_input(
                            &configuration.vocabulary.model_name,
                            &configuration.vocabulary.deck_name,
                        );
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
