use crate::anki_connect::client::AnkiClient;
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
        Command::AddCardModel { name: deck_name } => {
            let client = AnkiClient::default();
            client.create_model(&deck_name).await?;
        }
        Command::AddCardDeck { name: deck_name } => {
            let client = AnkiClient::default();
            client.create_deck(&deck_name).await?;
        }
    }

    Ok(())
}
