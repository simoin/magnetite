mod dashmap;
mod redis;

use crate::error::Result;
use crate::{Key, Value};

/// Set of method for basic storage providers to implement.
#[async_trait::async_trait]
pub trait Store: Send + Sync {
    /// Set a key-value pair, if the key already exist, value should be overwritten
    async fn set(&self, key: Key, value: Value) -> Result<()>;

    /// Get a value for specified key, it should result in None if the value does not exist
    async fn get(&self, key: Key) -> Result<Option<Value>>;

    /// Delete the key from storage, if the key doesn't exist, it shouldn't return an error
    async fn delete(&self, key: Key) -> Result<()>;
}
