use actix_web::ResponseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("xml parser: {0}")]
    XmlParseError(#[from] libxml::parser::XmlParseError),
}

impl ResponseError for Error {}
