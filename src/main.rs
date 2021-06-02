use actix_web::{App, HttpServer};
use magnetite_rs::cache::RssCache;
use magnetite_rs::middleware::Cache;
use magnetite_rs::rss_service;
use std::sync::Mutex;
use actix_service::Service;
use futures::FutureExt;
use actix_web::web::Data;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .data(Mutex::new(RssCache::new()))
            .wrap_fn(|req, srv| {
                // if let Some(cache) = req.app_data::<Data<Mutex<RssCache>>>() {
                //     println!("cache len: {}", cache.lock().unwrap().channel.len());
                // }
                // let cache = req.app_data::<Mutex<RssCache>>().unwrap().to_owned();

                srv.call(req).map(|res| {
                    res
                })
            })
            .wrap(Cache)
            .service(rss_service())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
