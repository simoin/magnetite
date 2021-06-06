use std::sync::Arc;

pub use redis::{ConnectionAddr, ConnectionInfo};

pub use storage::Storage;
pub use store::dashmap::DashMapActor;
pub use store::redis::RedisActor;

mod actor;
mod error;
mod storage;
mod store;

type Key = Arc<[u8]>;
type Value = Arc<[u8]>;
