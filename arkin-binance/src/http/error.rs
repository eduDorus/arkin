use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

/// Unsuccesful response from the Binance API.
#[derive(Debug)]
pub enum ClientError {
    /// API server error complying with the error schema.
    Structured(HttpError<BinanceApiError>),
    /// API server error not complying with the error schema.
    Raw(HttpError<String>),
}

/// Generic Http Error
#[derive(Debug)]
pub struct HttpError<T> {
    /// Http status code
    pub status_code: u16,
    /// Response body content
    pub data: T,
    /// Response headers
    pub headers: HashMap<String, String>,
}

impl<T> HttpError<T> {
    pub fn new(status_code: u16, data: T, headers: HashMap<String, String>) -> Self {
        Self {
            status_code,
            data,
            headers,
        }
    }
}

/// Structured Binance server error
#[derive(Deserialize, Debug)]
pub struct BinanceApiError {
    /// Error code
    ///
    /// [API Documentation](https://developers.binance.com/docs/margin_trading/error-code)
    #[serde(rename(deserialize = "code"))]
    pub code: i16,

    ///Error description
    #[serde(rename(deserialize = "msg"))]
    pub message: String,
}

// ThisError
#[derive(Error, Debug)]
pub enum BinanceHttpClientError {
    #[error("Invalid API secret")]
    InvalidApiSecret,
    #[error("Invalid PEM key")]
    InvalidPemKey(#[from] ed25519_dalek::pkcs8::Error),
    #[error("Signature error: {0}")]
    SignatureError(#[from] signature::Error),
    #[error("Failed to send request: {0}")]
    Send(#[from] reqwest::Error),
    #[error("Failed to parse url: {0}")]
    UrlParse(#[from] url::ParseError),
}
