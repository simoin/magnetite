use std::error::Error;

use actix_web::{http::StatusCode, ResponseError};
use thiserror::Error;

/// Error type that will be returned from all fallible methods of actix_storage.
///
/// implementers should generally use Custom variant for their own errors.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("RedisError: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("StorageError: Method not supported for the storage backend provided")]
    MethodNotSupported,
    #[error("StorageError: Serialization failed")]
    SerializationError,
    #[error("StorageError: Deserialization failed")]
    DeserializationError,
    #[error("StorageError: {0}")]
    Custom(Box<dyn Error + Send>),
}

impl StorageError {
    /// Shortcut method to construct Custom variant
    pub fn custom<E>(err: E) -> Self
    where
        E: 'static + Error + Send,
    {
        Self::Custom(Box::new(err))
    }
}

impl ResponseError for StorageError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

pub type Result<T> = std::result::Result<T, StorageError>;
