use crate::query::QueryClient;

mod kanji;
mod query;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = QueryClient::from_env()?;
    let kanji = client.list_kanji().await?;

    println!("{:#?}", kanji);

    Ok(())
}
