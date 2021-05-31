use actix_web::{App, HttpServer};
use magnetite_rs::rss_service;
use magnetite_rs::cache::RssCache;
use magnetite_rs::middleware::Cache;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new()
        .data(RssCache::new())
        .wrap(Cache)
        .service(rss_service()))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
