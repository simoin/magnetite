use crate::config::CACHE_EXPIRE;
use crate::error::Error;

use chrono::Utc;
use lru::LruCache;
use rss::Channel;

use std::future::Future;

pub struct RssCache(LruCache<String, CachedChannel>);
// pub struct RssCache {
//     pub channel: LruCache<String, CachedChannel>,
//     // resp: LruCache<String, CachedResponse>,
// }

pub struct CachedChannel {
    channel: Channel,
    expire: i64,
}

// struct CachedResponse {
//     body: String,
//     expire: i64,
// }

impl CachedChannel {
    fn new(channel: &Channel) -> Self {
        CachedChannel {
            channel: channel.to_owned(),
            expire: Utc::now().timestamp() + CACHE_EXPIRE as i64,
        }
    }
}

// impl CachedResponse {
//     fn new(body: &str) -> Self {
//         CachedResponse {
//             body: body.to_owned(),
//             expire: Utc::now().timestamp() + CACHE_EXPIRE as i64,
//         }
//     }
// }

impl RssCache {
    pub fn new() -> Self {
            RssCache(LruCache::new(20))
    }

    pub async fn try_get_channel<'a, F>(
        &mut self,
        key: &'a String,
        f: impl Fn(&'a str) -> F,
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

    pub fn get_channel(&mut self, key: &String) -> Option<Channel> {
        if let Some(value) = self.0.get(key) {
            if Utc::now().timestamp() < value.expire {
                return Some(value.channel.to_owned());
            }
        }
        None
    }

    pub fn set_channel(&mut self, key: &String, channel: &Channel) {
        // let mut cache = self.channel.write().unwrap();
        let value = CachedChannel::new(channel);
        self.0.put(key.to_owned(), value);
        // cache.put(key.to_owned(), value);
    }

    // pub async fn try_get_resp<'a, F>(&mut self, key: &'a String, f: impl Fn(&'a str) -> F) -> F::Output
    // where
    //     F: Future<Output = std::result::Result<String, Error>> + 'a,
    // {
    //     if let Some(resp) = self.get_resp(key) {
    //         return Ok(resp);
    //     }
    //
    //     let resp = f(key).await?;
    //     self.set_resp(key, &resp);
    //     Ok(resp)
    // }
    //
    // pub fn get_resp(&mut self, key: &String) -> Option<String> {
    //     // let mut cache = self.resp.write().unwrap();
    //     if let Some(value) =  self.resp.get(key) {
    //         if Utc::now().timestamp() < value.expire {
    //             return Some(value.body.to_owned());
    //         }
    //     }
    //     None
    // }
    //
    // pub fn set_resp(&mut self, key: &String, resp: &str) {
    //     // let mut cache = self.resp.write().unwrap();
    //     let value = CachedResponse::new(resp);
    //     self.resp.put(key.to_owned(), value);
    // }
}
