use thiserror::Error;

#[derive(Debug, Error)]
pub enum IoError {
    #[error("io error: {0}")]
    Io(#[from] tokio::io::Error),
    #[error("serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("cache not found")]
    CacheDirectoryNotFound,
    #[error("cache item not found")]
    CacheItemNotFound,
}

/// A container that can read and write cached data to the file system
pub struct FilesystemCache<'a> {
    pub cache_dir: &'a str,
}

impl<'a> FilesystemCache<'a> {
    pub async fn new(cache_dir: &'a str) -> Result<Self, IoError> {
        if tokio::fs::metadata(cache_dir).await.is_err() {
            return Err(IoError::CacheDirectoryNotFound);
        }
        Ok(Self { cache_dir })
    }

    /// Write a serializable value to the cache.
    pub async fn insert<T>(&self, key: &str, value: T) -> Result<(), IoError>
    where
        T: serde::Serialize,
    {
        let path = format!("{}/{}.json", self.cache_dir, key);
        tokio::fs::write(path, serde_json::to_string_pretty(&value)?).await?;
        Ok(())
    }

    /// Read an item from the cache, and deserialize it into the expected type.
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, IoError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let path = format!("{}/{}.json", self.cache_dir, key);
        if tokio::fs::metadata(&path).await.is_err() {
            return Err(IoError::CacheItemNotFound);
        }
        let value = tokio::fs::read_to_string(path).await?;
        Ok(Some(serde_json::from_str(&value)?))
    }
}
