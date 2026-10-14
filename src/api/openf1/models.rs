//! OpenF1 data transfer object models for API communication and serialization.

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Query parameters for fetching OpenF1 sessions.
#[derive(Deserialize, Serialize, Debug, Clone, IntoParams)]
pub struct SessionQueryParams {
    pub year: Option<u32>,
}

/// Query parameters for session-scoped OpenF1 requests.
#[derive(Deserialize, Serialize, Debug, Clone, IntoParams)]
pub struct SessionOnlyQueryParams {
    pub session_key: u64,
}

/// Query parameters for fetching OpenF1 driver 3D location samples.
#[derive(Deserialize, Serialize, Debug, Clone, IntoParams)]
pub struct LocationQueryParams {
    pub session_key: u64,
    pub date_gt: Option<String>,
}

/// OpenF1 Session metadata model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1Session {
    pub session_key: u64,
    #[serde(default)]
    pub session_name: Option<String>,
    #[serde(default)]
    pub session_type: Option<String>,
    #[serde(default)]
    pub date_start: Option<String>,
    #[serde(default)]
    pub date_end: Option<String>,
    #[serde(default)]
    pub gmt_offset: Option<String>,
    #[serde(default)]
    pub circuit_short_name: Option<String>,
    #[serde(default)]
    pub circuit_key: Option<u64>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub country_key: Option<u64>,
    #[serde(default)]
    pub country_name: Option<String>,
    #[serde(default)]
    pub year: Option<u32>,
    #[serde(default)]
    pub is_live: Option<bool>,
    #[serde(default)]
    pub is_cancelled: Option<bool>,
}

/// OpenF1 Driver metadata model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1Driver {
    pub session_key: u64,
    #[serde(default)]
    pub meeting_key: Option<u64>,
    pub driver_number: u32,
    #[serde(default)]
    pub broadcast_name: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub name_acronym: Option<String>,
    #[serde(default)]
    pub team_name: Option<String>,
    #[serde(default)]
    pub team_colour: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub headshot_url: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
}

/// OpenF1 Lap telemetry record model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1Lap {
    pub lap_number: u32,
    pub driver_number: u32,
    #[serde(default)]
    pub lap_duration: Option<f64>,
    #[serde(default)]
    pub duration_sector_1: Option<f64>,
    #[serde(default)]
    pub duration_sector_2: Option<f64>,
    #[serde(default)]
    pub duration_sector_3: Option<f64>,
    #[serde(default)]
    pub is_pit_out_lap: Option<bool>,
    #[serde(default)]
    pub stint: Option<u32>,
    #[serde(default)]
    pub compound: Option<String>,
    #[serde(default)]
    pub date_start: Option<String>,
}

/// OpenF1 High-frequency car telemetry sample.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1CarData {
    pub driver_number: u32,
    pub speed: f64,
    pub throttle: f64,
    pub brake: f64,
    pub rpm: u32,
    pub drs: u32,
    #[serde(default)]
    pub date: Option<String>,
}

/// OpenF1 3D location sample model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1Location {
    pub session_key: u64,
    pub driver_number: u32,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub z: Option<f64>,
    #[serde(default)]
    pub date: Option<String>,
}

/// Baked normalized track geometry point (0.0 - 1.0 scale).
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct NormalizedTrackPoint {
    pub x: f64,
    pub y: f64,
}

/// Compact normalized GPS coordinate sample for a driver.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct DriverGpsSample {
    pub driver_number: u32,
    pub t: f64,
    pub x: f64,
    pub y: f64,
}

/// OpenF1 Position model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1Position {
    pub session_key: u64,
    pub driver_number: u32,
    pub position: u32,
    #[serde(default)]
    pub date: Option<String>,
}

/// OpenF1 Interval model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1Interval {
    pub session_key: u64,
    pub driver_number: u32,
    #[serde(default)]
    pub gap_to_leader: Option<f64>,
    #[serde(default)]
    pub interval: Option<f64>,
    #[serde(default)]
    pub date: Option<String>,
}

/// OpenF1 Stint model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1Stint {
    #[serde(default)]
    pub session_key: Option<u64>,
    #[serde(default)]
    pub driver_number: Option<u32>,
    #[serde(default)]
    pub stint_number: Option<u32>,
    #[serde(default)]
    pub compound: Option<String>,
    #[serde(default)]
    pub tyre_age_at_start: Option<u32>,
    #[serde(default)]
    pub lap_start: Option<u32>,
    #[serde(default)]
    pub lap_end: Option<u32>,
}

/// OpenF1 Race Control event model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1RaceControl {
    pub session_key: u64,
    pub category: String,
    pub flag: Option<String>,
    pub message: Option<String>,
    pub scope: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
}

/// OpenF1 Weather sample model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1Weather {
    pub session_key: u64,
    pub air_temperature: f64,
    pub track_temperature: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub rainfall: u32,
    #[serde(default)]
    pub date: Option<String>,
}

/// Single driver state in live-state payload.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct LiveDriverState {
    pub driver_number: u32,
    pub tla: String,
    pub team: String,
    pub position: u32,
    pub gap: String,
    pub interval: String,
    pub x: f64,
    pub y: f64,
    pub compound: String,
    pub tyre_age: u32,
    #[serde(default)]
    pub pit_stop_count: u32,
    #[serde(default)]
    pub current_lap: u32,
    #[serde(default)]
    pub last_lap_time: Option<f64>,
    #[serde(default)]
    pub drs_active: bool,
}

/// Compressed Live State Response.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct LiveStateResponse {
    pub session_key: u64,
    pub timestamp_ms: u64,
    pub track_status: String,
    pub track_temperature: f64,
    pub air_temperature: f64,
    #[serde(default)]
    pub dynamic_odi: Option<f64>,
    #[serde(default)]
    pub odi_rating: Option<String>,
    pub drivers: Vec<LiveDriverState>,
}

/// Request payload for replaying OpenF1 telemetry into a simulation `RaceState`.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct ReplayStateRequest {
    pub session_key: u64,
    pub ego_driver_number: u32,
    pub current_lap: u32,
}
