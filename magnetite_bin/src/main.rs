use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use simple_logger::SimpleLogger;
use structopt::StructOpt;

use app_config::AppConfig;
use magnetite_core::gcores;

use crate::app_config::{config_path, Opt};

mod app_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // TODO use log4rs
    SimpleLogger::new().init().unwrap();

    let config = {
        let opt: Opt = Opt::from_args();
        eprintln!("opt = {:#?}", opt);
        let config_path = if let Some(config_path) = opt.config {
            config_path
        } else {
            config_path().unwrap()
        };
        AppConfig::from(config_path).unwrap()
    };
    eprintln!("settings = {:#?}", config);

    let addr = config.address();

    let app_state = config.into_state();

    let storage = app_state.storage().await;
    let app_state = Data::new(app_state);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(storage.clone())
            .service(web::scope("/").service(gcores::gcores_handle))
    })
    .bind(&addr)?
    .run()
    .await
}
