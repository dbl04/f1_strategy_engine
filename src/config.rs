//! Environment configuration parsing for the web server listener.

use std::env;

/// Server network binding configuration settings.
#[derive(Debug, Clone)]
pub struct Config {
    /// Host IP interface address to bind (defaults to `"0.0.0.0"`).
    pub host: String,
    /// TCP listening port number (defaults to `3000`).
    pub port: u16,
}

impl Config {
    /// Loads configuration values from environment variables (`HOST` and `PORT`),
    /// falling back to production defaults (`0.0.0.0:3000`) if unspecified.
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        Self { host, port }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}

