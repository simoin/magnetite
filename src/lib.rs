#[cfg(any(feature = "lru-cache", feature = "redis-cache"))]
mod cache;
mod config;
mod error;
mod http;
mod services;
mod sites;
mod util;

use http::CLIENT;
pub use services::rss_service;
use sites::gcores::gcores;
