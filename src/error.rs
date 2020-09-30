use actix_web::body::Body;
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("xml parser: {0}")]
    XmlParseError(#[from] libxml::parser::XmlParseError),
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<Body> {
        match self {
            Error::Reqwest(_) => HttpResponse::InternalServerError().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
