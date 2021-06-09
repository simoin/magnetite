use actix::{Actor, Context, Handler, ResponseActFuture, WrapFuture};
use redis::{aio::ConnectionManager, AsyncCommands, IntoConnectionInfo};

use crate::{
    actor::{StoreRequest, StoreResponse, CACHE_EXPIRE},
    error::{Result, StorageError},
};

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
    expire: usize,
}

pub struct RedisActorBuilder {
    url: Option<String>,
    expire: Option<usize>,
}

impl RedisActorBuilder {
    pub fn conn_info(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn expire(mut self, expire: usize) -> Self {
        self.expire = Some(expire);
        self
    }

    pub async fn finish(self) -> Result<RedisActor> {
        if self.url.is_none() {
            panic!("redis url is none")
        }
        let client = redis::Client::open(self.url.unwrap())?;
        let conn = client.get_tokio_connection_manager().await?;
        Ok(RedisActor {
            conn,
            expire: self.expire.unwrap_or(CACHE_EXPIRE),
        })
    }
}

impl RedisActor {
    pub fn new() -> RedisActorBuilder {
        RedisActorBuilder {
            url: None,
            expire: None,
        }
    }

    pub async fn connect<T: IntoConnectionInfo>(conn_info: T) -> Result<Self> {
        let client = redis::Client::open(conn_info)?;
        let conn = client.get_tokio_connection_manager().await?;
        Ok(RedisActor {
            conn,
            expire: CACHE_EXPIRE,
        })
    }
}

impl Actor for RedisActor {
    type Context = Context<Self>;
}

impl Handler<StoreRequest> for RedisActor {
    type Result = ResponseActFuture<Self, StoreResponse>;

    fn handle(&mut self, msg: StoreRequest, _: &mut Self::Context) -> Self::Result {
        let conn = self.conn.clone();
        let expire = self.expire;
        Box::pin(async move { msg_handle(conn, expire, msg).await }.into_actor(self))
    }
}

async fn msg_handle(
    mut conn: ConnectionManager,
    expire: usize,
    msg: StoreRequest,
) -> StoreResponse {
    match msg {
        StoreRequest::Set(key, value) => {
            let full_key = get_full_key(key);
            let res = conn.set_ex(full_key, value.as_ref(), expire).await;
            StoreResponse::Set(res.map_err(|err| StorageError::RedisError(err)))
        }
        StoreRequest::Get(key) => {
            let full_key = get_full_key(key);
            let res = conn.get(full_key).await;
            StoreResponse::Get(
                res.map(|val: Vec<u8>| {
                    if !val.is_empty() {
                        Some(val.into())
                    } else {
                        None
                    }
                })
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

#[cfg(test)]
mod redis_test {
    use std::time::Duration;

    use actix::Actor;
    use actix_rt::time::sleep;
    use redis::{ConnectionAddr, ConnectionInfo};

    use crate::error::Result;
    use crate::storage::Storage;
    use crate::store::redis::RedisActor;

    #[test]
    fn test() {
        let system = actix_rt::System::new();
        let store = system.block_on(async {
            let redis = RedisActor::new()
                .conn_info(ConnectionInfo {
                    addr: ConnectionAddr::Tcp("192.168.31.127".to_string(), 6380).into(),
                    db: 1,
                    username: None,
                    passwd: None,
                })
                .expire(1)
                .finish()
                .await
                .unwrap();
            redis.start()
        });
        let storage = Storage::new(store);

        system.block_on(async move {
            let key = "key";
            let value = "value".to_string();

            assert!(storage.set(key.as_bytes(), &value).await.is_ok());

            let get_res = storage.get(key).await;
            assert!(get_res.is_ok());
            assert_eq!(get_res.unwrap(), Some(value.clone()));

            assert!(storage.delete(key.as_bytes()).await.is_ok());
            let get_res: Result<Option<String>> = storage.get(key).await;
            assert_eq!(get_res.unwrap(), None);

            assert!(storage.set(key.as_bytes(), &value).await.is_ok());
            sleep(Duration::from_secs(2)).await;
            let get_res: Result<Option<String>> = storage.get(key).await;
            assert_eq!(get_res.unwrap(), None);
        });
    }
}
