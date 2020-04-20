pub mod auth;
pub mod datastore;

use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{self};

#[derive(Serialize, Deserialize, Debug)]
struct ResponseError {
    error: Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub code: u16,
    pub message: String,
    pub status: String,
}

impl Error {
    fn new(status_code: StatusCode, msg: String) -> Self {
        Error {
            code: status_code.as_u16(),
            message: msg,
            status: status_code.canonical_reason().unwrap().to_string(),
        }
    }
}

impl Into<String> for Error {
    fn into(self) -> String {
        format!("{} ({})", self.message, self.code)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("JSON_ERROR: {} (l{} : c{})", err, err.line(), err.column()),
        )
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        let status = match err.status() {
            Some(s) => s.as_u16(),
            None => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };

        let url = match err.url() {
            Some(u) => u.as_str(),
            None => "no url available",
        };

        Error {
            code: status,
            message: format!("{} (url: {})", err, url),
            status: status.to_string(),
        }
    }
}
