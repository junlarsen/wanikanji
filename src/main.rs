use crate::query::QueryClient;
use clap::Parser;

mod kanji;
mod query;

#[derive(clap::Parser)]
pub struct Options {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
    QueryKanji,
    CreateCardType,
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
        Command::CreateCardType => todo!(),
    }

    Ok(())
}
