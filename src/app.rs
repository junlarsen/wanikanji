use crate::anki_connect::client::{AnkiClient, AnkiError};
use crate::io::FilesystemCache;
use crate::kanji::ApiKanjiMessage;
use crate::query::QueryClient;
use crate::vocabulary::ApiVocabularyMessage;
use crate::Configuration;

/// Handle `wanikanji query-kanji` command
pub async fn handle_query_kanji(
    cache: &FilesystemCache<'_>,
    wanikani_client: &QueryClient,
) -> anyhow::Result<()> {
    let kanji = wanikani_client.list_kanji().await?;
    cache.insert("kanji", &kanji).await?;
    Ok(())
}

/// Handle `wanikanji query-vocabulary` command
pub async fn handle_query_vocabulary(
    cache: &FilesystemCache<'_>,
    wanikani_client: &QueryClient,
) -> anyhow::Result<()> {
    let vocabulary = wanikani_client.list_vocabulary().await?;
    cache.insert("vocabulary", &vocabulary).await?;
    Ok(())
}

/// Handle `wanikanji create-kanji-deck` command
pub async fn handle_create_kanji_deck(
    anki_client: &AnkiClient<'_>,
    configuration: &Configuration,
) -> anyhow::Result<()> {
    anki_client
        .create_kanji_model(
            &configuration.kanji.model_name,
            &configuration.kanji.model_template_name,
        )
        .await?;
    anki_client
        .create_deck(&configuration.kanji.deck_name)
        .await?;
    Ok(())
}

/// Handle `wanikanji create-vocabulary-deck` command
pub async fn handle_create_vocabulary_deck(
    anki_client: &AnkiClient<'_>,
    configuration: &Configuration,
) -> anyhow::Result<()> {
    anki_client
        .create_vocabulary_model(
            &configuration.vocabulary.model_name,
            &configuration.vocabulary.model_template_name,
        )
        .await?;
    anki_client
        .create_deck(&configuration.vocabulary.deck_name)
        .await?;
    Ok(())
}

/// Handle `wanikanji update-model-styling` command
pub async fn handle_update_model_styling(
    anki_client: &AnkiClient<'_>,
    configuration: &Configuration,
) -> anyhow::Result<()> {
    anki_client
        .update_model_styling(&configuration.kanji.model_name)
        .await?;
    anki_client
        .update_model_styling(&configuration.vocabulary.model_name)
        .await?;
    Ok(())
}

/// Handle `wanikanji update-model-templates` command
pub async fn handle_update_model_templates(
    anki_client: &AnkiClient<'_>,
    configuration: &Configuration,
) -> anyhow::Result<()> {
    anki_client
        .update_model_templates(&configuration.kanji)
        .await?;
    anki_client
        .update_model_templates(&configuration.vocabulary)
        .await?;
    Ok(())
}

/// Handle `wanikanji install-kanji` command
pub async fn handle_install_kanji(
    cache: &FilesystemCache<'_>,
    anki_client: &AnkiClient<'_>,
    configuration: &Configuration,
) -> anyhow::Result<()> {
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
                                if err.contains("cannot create note because it is a duplicate") =>
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
    Ok(())
}

/// Handle `wanikanji install-vocabulary` command
pub async fn handle_install_vocabulary(
    cache: &FilesystemCache<'_>,
    anki_client: &AnkiClient<'_>,
    configuration: &Configuration,
) -> anyhow::Result<()> {
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
                                if err.contains("cannot create note because it is a duplicate") =>
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
            tracing::error!("you must fetch vocabulary information before installing to deck")
        }
    }
    Ok(())
}
