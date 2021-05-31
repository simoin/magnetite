#[cfg(any(feature = "lru-cache", feature = "redis-cache"))]
pub mod cache;
mod config;
mod error;
mod http;
mod services;
mod sites;
mod util;
pub mod middleware;

use http::CLIENT;
pub use services::rss_service;
use sites::gcores::gcores;
use crate::cache::RssCache;

pub struct AppState {
    pub cache: RssCache,
}