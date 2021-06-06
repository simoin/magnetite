use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AppConfig {
    server: Server,
    cache: Cache,
    env: HashMap<String, String>,
    logger_level: String,
    proxy: String,
}

#[derive(Serialize, Deserialize)]
enum CacheType {
    Redis(String),
    Memory,
}

#[derive(Serialize, Deserialize)]
struct Cache {
    expire: usize,
    r#type: CacheType,
}

#[derive(Serialize, Deserialize)]
struct Server {
    listen: String,
    port: u16,
}
