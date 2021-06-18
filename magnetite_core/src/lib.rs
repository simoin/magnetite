use actix_web::{web, Scope};

use http::CLIENT;
pub use middleware::Cache;
use sites::gcores;

mod error;
mod http;
mod middleware;
mod sites;
pub mod state;
mod util;
mod xpath;

pub fn scope() -> Scope {
    web::scope("/").service(gcores::gcores_handle)
}
