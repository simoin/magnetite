use std::sync::Arc;

use actix::dev::MessageResponse;
use actix::{Actor, Addr, Handler, SyncArbiter, SyncContext};
use dashmap::DashMap;

use crate::actor::{StoreRequest, StoreResponse};
use crate::{Key, Value};

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
}

impl DashMapActor {
    pub fn new() -> Self {
        DashMapActor::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        DashMapActor {
            map: DashMap::with_capacity(capacity).into(),
        }
    }

    pub fn start_default(threads: usize) -> Addr<Self> {
        let storage = DashMapActor::default();
        SyncArbiter::start(threads, move || storage.clone())
    }

    pub fn start(self, threads: usize) -> Addr<Self> {
        SyncArbiter::start(threads, move || self.clone())
    }
}

impl Actor for DashMapActor {
    type Context = SyncContext<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        println!("Actor is alive");
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        println!("Actor is stopped");
    }
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
                // TODO check expire
                let value = self.map.get(&key).map(|val| val.bytes.clone());
                StoreResponse::Get(Ok(value))
            }
            StoreRequest::Delete(key) => {
                self.map.remove(&key);
                StoreResponse::Delete(Ok(()))
            }
        }
    }
}
