use std::collections::HashMap;

use magnetite_cache::*;

pub struct AppState {
    pub redis: Option<String>,
    pub cache_expire: usize,
    pub env: HashMap<String, String>,
}

impl AppState {
    pub async fn storage(&self) -> Storage {
        if let Some(redis_url) = &self.redis {
            redis_storage(redis_url.parse().unwrap(), Some(self.cache_expire)).await
        } else {
            dashmap_storage(self.cache_expire)
        }
    }
}
