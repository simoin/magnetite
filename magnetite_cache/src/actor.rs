use actix::{
    dev::{MessageResponse, OneshotSender, ToEnvelope},
    Actor, Addr, Handler, Message,
};

use crate::error::{Result, StorageError};
use crate::store::Store;
use crate::{Key, Value};

pub(crate) static CACHE_EXPIRE: usize = 5 * 60;

#[derive(Debug, Message)]
#[rtype(StoreResponse)]
pub enum StoreRequest {
    Get(Key),
    Set(Key, Value),
    Delete(Key),
}

pub enum StoreResponse {
    Get(Result<Option<Value>>),
    Set(Result<()>),
    Delete(Result<()>),
}

impl<A: Actor> MessageResponse<A, StoreRequest> for StoreResponse {
    fn handle(
        self,
        _: &mut <A as Actor>::Context,
        tx: Option<OneshotSender<<StoreRequest as Message>::Result>>,
    ) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}

#[async_trait::async_trait]
impl<T> Store for Addr<T>
where
    T: Actor + Handler<StoreRequest> + Sync + Send,
    T::Context: ToEnvelope<T, StoreRequest>,
{
    async fn set(&self, key: Key, value: Value) -> Result<()> {
        match self
            .send(StoreRequest::Set(key, value))
            .await
            .map_err(StorageError::custom)?
        {
            StoreResponse::Set(val) => val,
            _ => panic!(),
        }
    }

    async fn get(&self, key: Key) -> Result<Option<Value>> {
        match self
            .send(StoreRequest::Get(key))
            .await
            .map_err(StorageError::custom)?
        {
            StoreResponse::Get(val) => val,
            _ => panic!(),
        }
    }

    async fn delete(&self, key: Key) -> Result<()> {
        match self
            .send(StoreRequest::Delete(key))
            .await
            .map_err(StorageError::custom)?
        {
            StoreResponse::Delete(val) => val,
            _ => panic!(),
        }
    }
}
