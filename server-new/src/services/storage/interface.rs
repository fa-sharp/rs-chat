//! Storage interface

use std::path::Path;

use futures::future::BoxFuture;
use tokio::io::AsyncRead;

use super::error::StorageResult;

/// Trait representing an underlying storage to manage files for LLM chats and responses
pub trait StorageEngine: Send + Sync {
    fn create<'r>(
        &'r self,
        path: &'r Path,
        reader: &'r mut (dyn AsyncRead + Unpin + Send),
    ) -> BoxFuture<'r, StorageResult<usize>>;
    fn exists<'r>(&'r self, path: &'r Path) -> BoxFuture<'r, StorageResult<bool>>;
    fn delete<'r>(&'r self, path: &'r Path) -> BoxFuture<'r, StorageResult<()>>;
    fn signed_url(&self, #[allow(unused)] path: &Path) -> StorageResult<Option<String>> {
        Ok(None)
    }
}
