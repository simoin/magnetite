use crate::gcores;
use actix_web::{web, Scope};

pub fn rss_service() -> Scope {
    web::scope("/").service(gcores)
}
