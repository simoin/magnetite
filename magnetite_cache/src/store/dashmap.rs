use std::sync::Arc;

use actix::{Actor, Addr, Handler, SyncArbiter, SyncContext};
use chrono::Utc;
use dashmap::DashMap;

use crate::{
    actor::{StoreRequest, StoreResponse, CACHE_EXPIRE},
    Key, Value,
};

#[derive(Debug)]
struct DashMapValue {
    bytes: Value,
    // utc tz
    create_at: i64,
}

impl DashMapValue {
    fn new(bytes: Value) -> Self {
        DashMapValue {
            bytes,
            create_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[derive(Clone, Default)]
pub struct DashMapActor {
    map: Arc<DashMap<Key, DashMapValue>>,
    // default: 5 * 60
    ttl: i64,
}

impl DashMapActor {
    pub fn new(ttl: i64) -> Self {
        DashMapActor {
            ttl,
            ..Default::default()
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        DashMapActor {
            map: DashMap::with_capacity(capacity).into(),
            ttl: CACHE_EXPIRE as i64,
        }
    }

    pub fn set_ttl(mut self, ttl: i64) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn start_default(threads: usize) -> Addr<Self> {
        let storage = DashMapActor::new(CACHE_EXPIRE as i64);
        SyncArbiter::start(threads, move || storage.clone())
    }

    pub fn start(self, threads: usize) -> Addr<Self> {
        SyncArbiter::start(threads, move || self.clone())
    }
}

impl Actor for DashMapActor {
    type Context = SyncContext<Self>;
}

impl Handler<StoreRequest> for DashMapActor {
    type Result = StoreResponse;

    fn handle(&mut self, msg: StoreRequest, _: &mut Self::Context) -> Self::Result {
        match msg {
            StoreRequest::Set(key, value) => {
                self.map
                    .entry(key)
                    .and_modify(|val| {
                        val.bytes = value.clone();
                        val.create_at = chrono::Utc::now().timestamp();
                    })
                    .or_insert_with(|| DashMapValue::new(value));
                StoreResponse::Set(Ok(()))
            }
            StoreRequest::Get(key) => {
                let value = self.map.get(&key).map_or(None, |val| {
                    if val.create_at + self.ttl > Utc::now().timestamp() {
                        Some(val.bytes.clone())
                    } else {
                        None
                    }
                });
                StoreResponse::Get(Ok(value))
            }
            StoreRequest::Delete(key) => {
                self.map.remove(&key);
                StoreResponse::Delete(Ok(()))
            }
        }
    }
}

#[cfg(test)]
mod dashmap_test {
    use crate::error::Result;
    use crate::storage::Storage;
    use crate::store::dashmap::DashMapActor;

    #[test]
    fn test() {
        let system = actix_rt::System::new();
        let store = system.block_on(async { DashMapActor::new(600).start(1) });
        let storage = Storage::new(store);

        system.block_on(async move {
            let key = "key";
            let value = "value".to_string();

            assert!(storage.set(key.as_bytes(), &value).await.is_ok());

            let get_res = storage.get(key).await;
            assert!(get_res.is_ok());
            assert_eq!(get_res.unwrap(), Some(value));

            assert!(storage.delete(key.as_bytes()).await.is_ok());
            let get_res: Result<Option<String>> = storage.get(key).await;
            assert_eq!(get_res.unwrap(), None);
        });
    }
}
