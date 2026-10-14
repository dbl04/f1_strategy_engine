//! OpenF1 live/historical telemetry API client, models, and ingestion handlers.

pub mod client;
pub mod fallback;
pub mod handlers;
pub mod models;
pub mod utils;

// Re-export client
pub use client::OpenF1Client;

// Re-export all models
pub use models::*;

// Re-export handlers
pub use handlers::*;

// Re-export utility functions
pub use utils::{get_fallback_track_points, parse_iso_to_epoch};

// Re-export fallback generators for testing or direct usage
pub use fallback::*;
