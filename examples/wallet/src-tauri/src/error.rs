//! # Error Handling
//!
//! Basic passing of error information from API calls to the frontend.

use std::fmt::{Display, Formatter, Result};

use http::status;
use serde::Serialize;
use tauri_plugin_http::reqwest;

// TODO: improve error handling

#[derive(Debug, Serialize)]
#[allow(clippy::module_name_repetitions)]
pub struct AppError {
    message: String,
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        let status = err.status().unwrap_or(status::StatusCode::INTERNAL_SERVER_ERROR);
        let body = err.to_string();
        Self {
            message: format!("{status}:{body}"),
        }
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.message)
    }
}
