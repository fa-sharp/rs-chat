use std::path::Path;

use futures::future::BoxFuture;
use tokio::io::AsyncRead;

use super::error::StorageResult;

/// Trait representing an underlying storage to manage files for LLM chats and responses
pub trait StorageEngine: Send + Sync {
    fn create<'r>(
        &self,
        path: &Path,
        reader: &'r mut (dyn AsyncRead + Unpin + Send),
    ) -> BoxFuture<'r, StorageResult<usize>>;
    fn exists(&self, path: &Path) -> BoxFuture<'_, StorageResult<bool>>;
    fn delete(&self, path: &Path) -> BoxFuture<'_, StorageResult<()>>;
    fn signed_url(&self, path: &Path) -> StorageResult<String>;
}
