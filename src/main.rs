use actix_web::{App, HttpServer};

use actix_storage::{Format, Storage};
use actix_storage_dashmap::DashMapActor;
use magnetite_rs::rss_service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let rss_store = DashMapActor::with_capacity(20).start(4);
    let rss_storage = Storage::build()
        .store(rss_store)
        .format(Format::Json)
        .finish();

    HttpServer::new(move || {
        App::new()
            .app_data(rss_storage.clone())
            .service(rss_service())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
