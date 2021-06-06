use actix::{Actor, Context, Handler, ResponseActFuture, WrapFuture};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, ConnectionInfo};

use crate::actor::{StoreRequest, StoreResponse};
use crate::error::{Result, StorageError};

const SCOPE: [u8; 9] = *b"RSS_CACHE";

fn get_full_key<K>(key: K) -> Vec<u8>
where
    K: AsRef<[u8]>,
{
    [SCOPE.as_ref(), b":", key.as_ref()].concat()
}

#[derive(Clone)]
pub struct RedisActor {
    conn: ConnectionManager,
    // default: 5 * 60
    ttl: usize,
}

impl RedisActor {
    pub async fn connect(conn_info: ConnectionInfo) -> Result<Self> {
        let client = redis::Client::open(conn_info)?;
        let conn = client.get_tokio_connection_manager().await?;
        Ok(RedisActor { conn, ttl: 5 * 60 })
    }

    pub fn set_ttl(mut self, ttl: usize) -> Self {
        self.ttl = ttl;
        self
    }
}

impl Actor for RedisActor {
    type Context = Context<Self>;
}

impl Handler<StoreRequest> for RedisActor {
    type Result = ResponseActFuture<Self, StoreResponse>;

    fn handle(&mut self, msg: StoreRequest, _: &mut Self::Context) -> Self::Result {
        let conn = self.conn.clone();
        let ttl = self.ttl;
        Box::pin(async move { msg_handle(conn, ttl, msg).await }.into_actor(self))
    }
}

async fn msg_handle(mut conn: ConnectionManager, ttl: usize, msg: StoreRequest) -> StoreResponse {
    match msg {
        StoreRequest::Set(key, value) => {
            let full_key = get_full_key(key);
            let res = conn.set_ex(full_key, value.as_ref(), ttl).await;
            StoreResponse::Set(res.map_err(|err| StorageError::RedisError(err)))
        }
        StoreRequest::Get(key) => {
            let full_key = get_full_key(key);
            let res = conn.get(full_key).await;
            StoreResponse::Get(
                res.map(|val: Vec<u8>| Some(val.into()))
                    .map_err(|err| StorageError::RedisError(err)),
            )
        }
        StoreRequest::Delete(key) => {
            let full_key = get_full_key(key);
            let res = conn.del(full_key).await;
            StoreResponse::Delete(res.map_err(|err| StorageError::RedisError(err)))
        }
    }
}
