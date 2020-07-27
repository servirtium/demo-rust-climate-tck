use hyper::http;
use std::{fmt::Display, io, sync};

#[derive(Debug)]
pub enum ServirtiumError {
    InvalidMarkdownFormat,
    IoError(io::Error),
    PoisonedLock,
    InvalidStatusCode,
    NotConfigured,
    ReqwestError(reqwest::Error),
    InvalidHeaderName,
    InvalidHeaderValue,
    InvalidBody,
    HyperError(hyper::Error),
    ParseUriError,
    HttpError(http::Error),
    UnknownError,
}

impl std::error::Error for ServirtiumError {}

impl Display for ServirtiumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServirtiumError::InvalidMarkdownFormat => write!(f, "The markdown format was poisoned"),
            ServirtiumError::IoError(e) => write!(f, "IoError: {}", e),
            ServirtiumError::PoisonedLock => write!(f, "The lock was poisoned"),
            ServirtiumError::InvalidStatusCode => write!(f, "The status code is invalid"),
            ServirtiumError::NotConfigured => write!(f, "The server hasn't been configured"),
            ServirtiumError::ReqwestError(e) => write!(f, "reqwest error: {}", e),
            ServirtiumError::InvalidHeaderName => write!(f, "Invalid header name"),
            ServirtiumError::InvalidHeaderValue => write!(f, "Invalid header value"),
            ServirtiumError::InvalidBody => write!(f, "Invalid body"),
            ServirtiumError::HyperError(e) => write!(f, "Hyper error: {}", e),
            ServirtiumError::ParseUriError => write!(f, "Parse URI Error"),
            ServirtiumError::UnknownError => write!(f, "Unknown Servirtium Error"),
            ServirtiumError::HttpError(e) => write!(f, "Http Error: {}", e),
        }
    }
}

impl From<io::Error> for ServirtiumError {
    fn from(e: io::Error) -> Self {
        ServirtiumError::IoError(e)
    }
}

impl<T> From<sync::PoisonError<T>> for ServirtiumError {
    fn from(_: sync::PoisonError<T>) -> Self {
        ServirtiumError::PoisonedLock
    }
}

impl From<reqwest::Error> for ServirtiumError {
    fn from(e: reqwest::Error) -> Self {
        ServirtiumError::ReqwestError(e)
    }
}

impl From<hyper::header::InvalidHeaderName> for ServirtiumError {
    fn from(_: hyper::header::InvalidHeaderName) -> Self {
        ServirtiumError::InvalidHeaderName
    }
}

impl From<hyper::header::InvalidHeaderValue> for ServirtiumError {
    fn from(_: hyper::header::InvalidHeaderValue) -> Self {
        ServirtiumError::InvalidHeaderValue
    }
}

impl From<hyper::Error> for ServirtiumError {
    fn from(e: hyper::Error) -> Self {
        ServirtiumError::HyperError(e)
    }
}

impl From<http::Error> for ServirtiumError {
    fn from(e: http::Error) -> Self {
        ServirtiumError::HttpError(e)
    }
}
