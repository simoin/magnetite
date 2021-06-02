use chrono::Utc;
use rss::Channel;
use serde::{Deserialize, Serialize};

use crate::config::CACHE_EXPIRE;

#[derive(Serialize, Deserialize)]
pub struct CachedChannel {
    channel: Channel,
    expire: i64,
}

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