use std::sync::Arc;

use actix::Actor;
pub use redis::{ConnectionAddr, ConnectionInfo};

pub use storage::Storage;
use store::{dashmap::DashMapActor, redis::RedisActor};

mod actor;
pub mod error;
mod storage;
mod store;

type Key = Arc<[u8]>;
type Value = Arc<[u8]>;

pub fn dashmap_storage(cache_expire: usize) -> Storage {
    let store = DashMapActor::new(cache_expire as i64).start(2);
    Storage::new(store)
}

pub async fn redis_storage(url: String, cache_expire: Option<usize>) -> Storage {
    let store = RedisActor::new()
        .conn_info(url)
        .expire(cache_expire.unwrap())
        .finish()
        .await
        .unwrap()
        .start();
    Storage::new(store)
}
