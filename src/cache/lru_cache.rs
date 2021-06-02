use std::future::Future;

use chrono::Utc;
use dashmap::DashMap;
use rss::Channel;
use serde::{Deserialize, Serialize};

use crate::config::CACHE_EXPIRE;
use crate::error::Error;

pub struct RssCache {
    inner: DashMap<String, CachedChannel>,
}
// pub struct RssCache {
//     pub channel: LruCache<String, CachedChannel>,
//     // resp: LruCache<String, CachedResponse>,
// }

#[derive(Serialize, Deserialize)]
pub struct CachedChannel {
    channel: Channel,
    expire: i64,
}

// struct CachedResponse {
//     body: String,
//     expire: i64,
// }

impl CachedChannel {
    pub fn new(channel: &Channel) -> Self {
        CachedChannel {
            channel: channel.to_owned(),
            expire: Utc::now().timestamp() + CACHE_EXPIRE as i64,
        }
    }

    pub fn is_valid(&self) -> bool {
        Utc::now().timestamp() < self.expire
    }

    pub fn to_string(&self) -> String {
        self.channel.to_string()
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
        RssCache {
            inner: DashMap::with_capacity(20),
        }
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
        if let Some(value) = self.inner.get(key) {
            if Utc::now().timestamp() < value.expire {
                return Some(value.channel.to_owned());
            }
        }
        None
    }

    pub fn set_channel(&mut self, key: &String, channel: &Channel) {
        // let mut cache = self.channel.write().unwrap();
        let value = CachedChannel::new(channel);
        self.inner.insert(key.to_owned(), value);
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
