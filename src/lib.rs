#[cfg(any(feature = "lru-cache", feature = "redis-cache"))]
pub mod cache;
mod config;
mod error;
mod http;
pub mod middleware;
mod services;
mod sites;
mod util;

use crate::cache::RssCache;
use http::CLIENT;
pub use services::rss_service;
use sites::gcores::gcores;

pub struct AppState {
    pub cache: RssCache,
}
