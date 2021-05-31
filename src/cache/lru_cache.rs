use crate::config::CACHE_EXPIRE;
use crate::error::Error;

use chrono::Utc;
use lazy_static::lazy_static;
use lru::LruCache;
use rss::Channel;

use std::future::Future;
use std::sync::RwLock;


pub struct RssCache {
    channel: RwLock<LruCache<String, CachedChannel>>,
    resp: RwLock<LruCache<String, CachedResponse>>,
}

struct CachedChannel {
    channel: Channel,
    expire: i64,
}

struct CachedResponse {
    body: String,
    expire: i64,
}

impl CachedChannel {
    fn new(channel: &Channel) -> Self {
        let now = Utc::now().timestamp();
        CachedChannel {
            channel: channel.to_owned(),
            expire: now + CACHE_EXPIRE as i64,
        }
    }
}

impl CachedResponse {
    fn new(body: &str) -> Self {
        let now = Utc::now().timestamp();
        CachedResponse {
            body: body.to_owned(),
            expire: now + CACHE_EXPIRE as i64,
        }
    }
}

impl RssCache {
    pub fn new() -> Self {
        RssCache {
            channel: RwLock::new(LruCache::new(20)),
            resp: RwLock::new(LruCache::new(20)),
        }
    }

    pub async fn try_get_channel<'a, F>(
        &self, key: &'a String, f: impl Fn(&'a str) -> F,
    ) -> F::Output
    where
        F: Future<Output = std::result::Result<Channel, Error>> + 'a,
    {
        if let Some(channel) = self.get_channel(key) {
            return Ok(channel);
        }

        let channel = f(key).await?;
        self.set_channel(key, &channel);
        Ok(channel)
    }

    pub fn get_channel(&self, key: &String) -> Option<Channel> {
        let mut cache = self.channel.write().unwrap();
        if let Some(value) = (*cache).get(key) {
            if Utc::now().timestamp() < value.expire {
                return Some(value.channel.to_owned());
            }
        }
        None
    }

    pub fn set_channel(&self, key: &String, channel: &Channel) {
        let mut cache = self.channel.write().unwrap();
        let value = CachedChannel::new(channel);
        cache.put(key.to_owned(), value);
    }

    pub async fn try_get_resp<'a, F>(&self, key: &'a String, f: impl Fn(&'a str) -> F) -> F::Output
    where
        F: Future<Output = std::result::Result<String, Error>> + 'a,
    {
        if let Some(resp) = self.get_resp(key) {
            return Ok(resp);
        }

        let resp = f(key).await?;
        self.set_resp(key, &resp);
        Ok(resp)
    }

    pub fn get_resp(&self, key: &String) -> Option<String> {
        let mut cache = self.resp.write().unwrap();
        if let Some(value) = (*cache).get(key) {
            if Utc::now().timestamp() < value.expire {
                return Some(value.body.to_owned());
            }
        }
        None
    }

    pub fn set_resp(&self, key: &String, resp: &str) {
        let mut cache = self.resp.write().unwrap();
        let value = CachedResponse::new(resp);
        cache.put(key.to_owned(), value);
    }
}
