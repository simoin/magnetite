use actix_web::{App, HttpServer, web};
use actix::Actor;
use magnetite_cache::*;
use magnetite_core::gcores;

#[cfg(feature = "memory")]
fn rss_storage() -> Storage {
    let rss_store = DashMapActor::with_capacity(20).start(2);
    Storage::new(rss_store)
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
        let redis = RedisActor::new()
            .conn_info(connection_info)
            .with_ttl(600)
            .finish()
            .await.unwrap();
        redis.start()
    };
    Storage::new(rss_store)
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
