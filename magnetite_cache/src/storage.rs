use std::future::{ready, Ready};
use std::sync::Arc;

use actix_web::{error::ErrorInternalServerError, FromRequest, HttpRequest};
use actix_web::dev::Payload;

use crate::error::{Result, StorageError};
use crate::store::Store;

#[derive(Clone)]
pub struct Storage {
    store: Arc<dyn Store>,
}

impl Storage {
    pub fn new<T>(store: T) -> Self where T: 'static + Store {
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

impl FromRequest for Storage {
    type Config = ();
    type Error = actix_web::Error;
    type Future = Ready<std::result::Result<Self, actix_web::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(st) = req.app_data::<Storage>() {
            ready(Ok(st.clone()))
        } else {
            log::debug!(
                "Failed to construct Storage(actix-storage). \
                 Request path: {:?}",
                req.path(),
            );
            ready(Err(ErrorInternalServerError(
                "Storage is not configured, please refer to actix-storage documentation\
                for more information.",
            )))
        }
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
