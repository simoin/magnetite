mod app_config;
use simple_logger::SimpleLogger;
use actix_web::{web, App, HttpServer};
use config::Config;

use magnetite_cache::*;
use magnetite_core::gcores;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new().init().unwrap();

    #[cfg(feature = "memory")]
    let rss_storage = dashmap_storage(5);
    #[cfg(feature = "redis")]
    let rss_storage = {
        let conn_info = ConnectionInfo {
            addr: ConnectionAddr::Tcp("192.168.31.127".to_string(), 6380).into(),
            db: 1,
            username: None,
            passwd: None,
        };
        redis_storage(conn_info, Some(600)).await
    };

    HttpServer::new(move || {
        App::new()
            .app_data(rss_storage.clone())
            .service(web::scope("/").service(gcores::gcores_handle))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
