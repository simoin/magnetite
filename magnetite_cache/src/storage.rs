use std::sync::Arc;

use crate::error::{Result, StorageError};
use crate::store::Store;

pub struct Storage<T: Store> {
    store: Arc<T>,
}

impl<T: Store> Storage<T> {
    pub fn new(store: T) -> Self {
        Storage {
            store: Arc::new(store),
        }
    }

    pub async fn set<K, V>(&self, key: K, value: &V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: serde::Serialize,
    {
        self.store
            .set(key.as_ref().into(), serialize(value)?.into())
            .await
    }

    pub async fn get<K, V>(&self, key: K) -> Result<Option<V>>
    where
        K: AsRef<[u8]>,
        V: serde::de::DeserializeOwned,
    {
        let val = self.store.get(key.as_ref().into()).await?;
        val.map(|val| deserialize(val.as_ref())).transpose()
    }

    pub async fn delete<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>,
    {
        self.store.delete(key.as_ref().into()).await
    }
}

fn serialize<T>(value: &T) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    serde_json::to_vec(value).map_err(|_| StorageError::SerializationError)
}

fn deserialize<T>(value: &[u8]) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_slice(value).map_err(|_| StorageError::DeserializationError)
}
