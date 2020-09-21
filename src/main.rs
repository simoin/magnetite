use actix_web::{web, App, HttpServer, Scope};
use rsshub_rs::sites::*;

fn my_service() -> Scope {
    web::scope("/").route("/gcores", web::get().to(gcores))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(my_service())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
