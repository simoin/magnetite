use std::fmt::Debug;

use actix_web::ResponseError;
use log::error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("xml parser: {0}")]
    XmlParseError(#[from] libxml::parser::XmlParseError),
    #[error("cache: {0}")]
    CacheError(#[from] magnetite_cache::error::StorageError),
    #[error("xml operate: {0}")]
    LibXMLError(String),
}

impl ResponseError for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) trait CustomError<T> {
    fn custom_err(self, msg: &str) -> Result<T>;
}

impl<T, E: Debug> CustomError<T> for std::result::Result<T, E> {
    fn custom_err(self, msg: &str) -> Result<T> {
        self.map_err(|e| {
            error!(target: "libxml", "Error: {:#?}", e);
            Error::LibXMLError(msg.to_owned())
        })
    }
}
