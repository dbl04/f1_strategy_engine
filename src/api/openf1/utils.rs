//! Utility helpers for timestamp parsing and fallback track data loading.

use super::models::NormalizedTrackPoint;

static FALLBACK_TRACK_JSON: &str = include_str!("../../../assets/debug/clean_spike_location.json");

/// Loads the embedded fallback track geometry points from the bundled JSON asset.
pub fn get_fallback_track_points() -> Vec<NormalizedTrackPoint> {
    serde_json::from_str(FALLBACK_TRACK_JSON).unwrap_or_default()
}

/// Parses an ISO 8601 timestamp string into epoch seconds.
///
/// First attempts RFC 3339 parsing via `chrono`. Falls back to a rough manual
/// extraction for non-standard timestamp formats.
pub fn parse_iso_to_epoch(iso: &str) -> Option<u64> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        return Some(dt.timestamp().max(0) as u64);
    }
    if iso.len() < 19 {
        return None;
    }
    let year: u64 = iso[0..4].parse().ok()?;
    let month: u64 = iso[5..7].parse().ok()?;
    let day: u64 = iso[8..10].parse().ok()?;
    let hour: u64 = iso[11..13].parse().ok()?;
    let min: u64 = iso[14..16].parse().ok()?;
    let sec: u64 = iso[17..19].parse().ok()?;

    let days_since_epoch = (year.saturating_sub(1970)) * 365
        + (year.saturating_sub(1969)) / 4
        + (month.saturating_sub(1)) * 30
        + (day.saturating_sub(1));
    Some(days_since_epoch * 86400 + hour * 3600 + min * 60 + sec)
}
