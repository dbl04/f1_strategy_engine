//! Application error handling module providing strongly-typed error variants and Axum HTTP response conversion.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Application-wide error enumeration for API handlers and domain services.
#[derive(Debug)]
pub enum AppError {
    /// External OpenStreetMap Overpass API communication or parsing failure.
    OverpassError(String),
    /// Simulation domain execution or calculation failure.
    SimulationError(String),
    /// Bad user request payload or unparseable query parameters.
    InvalidInput(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::OverpassError(msg) => write!(f, "Overpass API error: {}", msg),
            AppError::SimulationError(msg) => write!(f, "Simulation error: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::OverpassError(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::SimulationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
