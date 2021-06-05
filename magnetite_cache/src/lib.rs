use std::sync::Arc;

mod actor;
mod error;
mod storage;
mod store;

type Key = Arc<[u8]>;
type Value = Arc<[u8]>;
