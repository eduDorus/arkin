use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug, Clone)]
pub struct WsResponse<R> {
    pub id: Option<String>,
    #[serde(default)]
    pub status: u16,
    #[serde(default)]
    pub result: Option<R>,
    #[serde(default)]
    pub error: Option<ApiError>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiError {
    pub code: i64,
    pub msg: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API Error {{ code: {}, message: \"{}\" }}", self.code, self.msg)
    }
}

impl std::error::Error for ApiError {}

#[derive(Deserialize, Debug, Clone)]
pub struct EmptyResult {}
