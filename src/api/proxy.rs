//! Proxy integration module for OpenStreetMap Overpass API circuit geometry fetching.

use crate::error::AppError;
use serde_json::Value;

/// Performs percent-encoding on a raw URL query string segment.
///
/// Encodes non-alphanumeric and non-unreserved ASCII characters into `%XX` hex format
/// to ensure safety when constructing HTTP query parameters.
///
/// # Arguments
/// * `s` - Raw string slice to percent-encode.
pub fn urlencoding(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                format!("{}", b as char)
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}

/// Asynchronously fetches 2D circuit geometry from OpenStreetMap's Overpass API.
///
/// Constructs an Overpass QL query searching for relation or way raceway elements matching
/// the given circuit `name`, queries `https://overpass-api.de`, and parses the resulting JSON.
///
/// # Arguments
/// * `name` - The search name of the circuit (e.g., `"Silverstone Circuit"`).
///
/// # Errors
/// Returns an [`AppError::OverpassError`] if the HTTP request fails or if the response JSON cannot be parsed.
pub async fn fetch_circuit_geometry(name: &str) -> Result<Value, AppError> {
    let overpass_query = format!(
        r#"[out:json][timeout:25];(relation["name"~"{}",i];way["highway"="raceway"]["name"~"{}",i];way["name"~"{}",i]["highway"];);(._;>;);out geom;"#,
        name, name, name
    );

    let url = format!(
        "https://overpass-api.de/api/interpreter?data={}",
        urlencoding(&overpass_query)
    );

    let resp = reqwest::get(&url)
        .await
        .map_err(|e| AppError::OverpassError(format!("Overpass API request failed: {}", e)))?;

    let json = resp.json::<Value>().await.map_err(|e| {
        AppError::OverpassError(format!("Failed to parse Overpass response: {}", e))
    })?;

    Ok(json)
}
