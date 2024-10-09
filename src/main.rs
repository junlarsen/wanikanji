use crate::anki_connect::client::AnkiClient;
use crate::io::FilesystemCache;
use crate::query::QueryClient;
use clap::Parser;
use config::Config;
use serde::Deserialize;

pub mod anki;
pub mod anki_connect;
pub mod app;
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
    #[clap(about = "Update Anki model styling to use the included CSS file")]
    UpdateModelStyling,
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
        Command::QueryKanji => app::handle_query_kanji(&cache, &wanikani_client).await?,
        Command::QueryVocabulary => app::handle_query_vocabulary(&cache, &wanikani_client).await?,
        Command::CreateKanjiDeck => {
            app::handle_create_kanji_deck(&anki_client, &configuration).await?
        }
        Command::CreateVocabularyDeck => {
            app::handle_create_vocabulary_deck(&anki_client, &configuration).await?
        }
        Command::InstallKanji => {
            app::handle_install_kanji(&cache, &anki_client, &configuration).await?
        }
        Command::InstallVocabulary => {
            app::handle_install_vocabulary(&cache, &anki_client, &configuration).await?
        }
        Command::UpdateModelStyling => {
            app::handle_update_model_styling(&anki_client, &configuration).await?
        }
    }
    Ok(())
}
