use lru::LruCache;

use crate::config::CACHE_EXPIRE;
use crate::error::Error;
use cfg_if::cfg_if;
use chrono::Utc;
use lazy_static::lazy_static;
use rss::Channel;
use std::future::Future;
use std::ops::Deref;
use std::sync::RwLock;

// TODO redis implement
lazy_static! {
    pub static ref CACHE: Cache = Cache(RwLock::new(LruCache::new(20)));
}

cfg_if! {
    if #[cfg(feature = "lru-cache")] {
        pub struct Cache(RwLock<LruCache<String, CachedItem>>);

        pub struct CachedItem{
            channel: Channel,
            expire: i64,
        }

        impl CachedItem {
            pub fn new(channel: &Channel) -> Self {
                let now = Utc::now().timestamp();
                CachedItem {
                    channel: channel.to_owned(),
                    expire: now + CACHE_EXPIRE as i64
                }
            }
        }
        impl Deref for Cache {
            type Target = RwLock<LruCache<String, CachedItem>>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl Cache {
            pub async fn try_get<'a, F>(&self, key: &'a String, f: impl Fn(&'a str) -> F) -> F::Output
                where F: Future<Output=std::result::Result<Channel, Error>> + 'a {
                if let Some(channel) = self.get(key) {
                    return Ok(channel);
                }

                let channel = f(key).await?;
                self.set(key, &channel);
                Ok(channel)
            }

            pub fn get(&self, key: &String) -> Option<Channel> {
                let mut cache = self.write().unwrap();
                if let Some(item) = (*cache).get(key) {
                    if Utc::now().timestamp() < item.expire {
                        return Some(item.channel.to_owned());
                    }
                }
                None
            }

            pub fn set(&self, key: &String, channel: &Channel) {
                let mut cache = self.write().unwrap();
                let item = CachedItem::new(channel);
                cache.put(key.to_owned(), item);
            }
        }
    } else {

        // pub struct Cache(RedisConnection);
        //
        // impl Cache {
        //     pub fn get(&self) {}
        //     pub fn set(&self) {}
        // }
    }
}