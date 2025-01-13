use thiserror::Error;
use hyper::StatusCode;
use std::error::Error;

#[derive(Error, Debug)]
pub enum YouTubeError {
    #[error("Account is closed")]
    AccountClosed,
    #[error("Account is terminated")]
    AccountTerminated,
    #[error("Subscriptions are private")]
    SubscriptionsPrivate,
    #[error("Not found")]
    NotFound,
    #[error("Ratelimited")]
    Ratelimited,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Unknown Status Code")]
    UnknownStatusCode(StatusCode),
    #[error("Parse error")]
    ParseError(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] hyper::Error),
    #[error("Legacy HTTP error: {0}")]
    LegacyHttpError(#[from] hyper_util::client::legacy::Error),
    #[error("Protobuf error: {0}")]
    ProtobufError(#[from] prost::DecodeError),
    #[error("Other error: {0}")]
    Other(Box<dyn Error + Send + Sync>),
}