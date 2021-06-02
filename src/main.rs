use std::sync::Mutex;

use actix_service::Service;
use actix_web::{App, HttpServer};
use actix_web::web::Data;
use futures::FutureExt;

use magnetite_rs::cache::RssCache;
use magnetite_rs::middleware::Cache;
use magnetite_rs::rss_service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let rss_cache = Data::new(Mutex::new(RssCache::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(rss_cache.clone())
            .wrap(Cache)
            .service(rss_service())
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
