use crate::config::CACHE_EXPIRE;
use chrono::Utc;
use rss::{Channel, ChannelBuilder, Item};

pub mod gcores;

fn channel(title: String, url: String, items: Vec<Item>) -> Channel {
    ChannelBuilder::default()
        .title(title)
        .link(url)
        .description("magnetite_rs")
        .language("zh-cn".to_string())
        .generator("magnetite_rs".to_string())
        .ttl((CACHE_EXPIRE / 60).to_string())
        .last_build_date(Utc::now().to_rfc2822())
        .items(items)
        .build()
        .unwrap()
}
