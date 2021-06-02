use actix_storage::{Format, Storage};
#[cfg(feature = "memory")]
use actix_storage_dashmap::DashMapActor;
#[cfg(feature = "redis")]
use actix_storage_redis::{ConnectionAddr, ConnectionInfo, RedisBackend};
use actix_web::{App, HttpServer};

use magnetite_rs::rss_service;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(feature = "memory")]
    let rss_store = DashMapActor::with_capacity(20).start(4);
    #[cfg(feature = "redis")]
    let rss_store = {
        let connection_info = ConnectionInfo {
            addr: ConnectionAddr::Tcp("192.168.31.127".to_string(), 6380).into(),
            db: 1,
            username: None,
            passwd: None,
        };
        RedisBackend::connect(connection_info)
            .await
            .expect("Redis connection failed")
    };
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
