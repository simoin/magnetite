use actix_storage::{Format, Storage};
#[cfg(feature = "memory")]
use actix_storage_dashmap::DashMapActor;
#[cfg(feature = "redis")]
use actix_storage_redis::{ConnectionAddr, ConnectionInfo, RedisBackend};
use actix_web::{web, App, HttpServer};

use magnetite_core::gcores;

// TODO use cfg-if
#[cfg(feature = "memory")]
fn rss_storage() -> Storage {
    let rss_store = DashMapActor::with_capacity(20).start(2);
    Storage::build()
        .store(rss_store)
        .format(Format::Json)
        .finish()
}

#[cfg(feature = "redis")]
async fn rss_storage() -> Storage {
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
    Storage::build()
        .expiry_store(rss_store)
        .format(Format::Json)
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[cfg(feature = "memory")]
    let rss_storage = rss_storage();
    #[cfg(feature = "redis")]
    let rss_storage = rss_storage().await;

    HttpServer::new(move || {
        App::new()
            .app_data(rss_storage.clone())
            .service(web::scope("/").service(gcores::gcores_handle))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
