//! OpenF1 live/historical telemetry API client and ingestion handlers.

use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::{CompetitorCar, EgoCar, RaceState, TireCompound, TrackParameters, TrackStatus};
use crate::error::AppError;

/// OpenF1 Session metadata model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1Session {
    pub session_key: u64,
    pub circuit_short_name: String,
    pub session_name: String,
    pub year: u32,
}

/// OpenF1 Lap telemetry record model.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct OpenF1Lap {
    pub lap_number: u32,
    pub driver_number: u32,
    #[serde(default)]
    pub lap_duration: Option<f64>,
    #[serde(default)]
    pub stint: Option<u32>,
    #[serde(default)]
    pub compound: Option<String>,
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

/// OpenF1 HTTP Client targeting `https://api.openf1.org/v1`.
#[derive(Debug, Clone, Default)]
pub struct OpenF1Client;

impl OpenF1Client {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_recent_sessions(&self) -> Result<Vec<OpenF1Session>, AppError> {
        let url = "https://api.openf1.org/v1/sessions?year=2024";
        let resp = reqwest::get(url)
            .await
            .map_err(|e| AppError::OverpassError(format!("OpenF1 API request failed: {}", e)))?;

        let sessions = resp
            .json::<Vec<OpenF1Session>>()
            .await
            .map_err(|e| AppError::OverpassError(format!("Failed to parse OpenF1 sessions: {}", e)))?;

        Ok(sessions)
    }

    pub async fn get_laps(
        &self,
        session_key: u64,
        driver_number: u32,
    ) -> Result<Vec<OpenF1Lap>, AppError> {
        let url = format!(
            "https://api.openf1.org/v1/laps?session_key={}&driver_number={}",
            session_key, driver_number
        );
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| AppError::OverpassError(format!("OpenF1 API request failed: {}", e)))?;

        let laps = resp
            .json::<Vec<OpenF1Lap>>()
            .await
            .map_err(|e| AppError::OverpassError(format!("Failed to parse OpenF1 laps: {}", e)))?;

        Ok(laps)
    }
}

/// Handler serving `GET /api/openf1/sessions`.
#[utoipa::path(
    get,
    path = "/api/openf1/sessions",
    responses(
        (status = 200, description = "List of recent F1 sessions from OpenF1 API", body = Vec<OpenF1Session>)
    )
)]
pub async fn get_openf1_sessions() -> Result<Json<Vec<OpenF1Session>>, AppError> {
    let client = OpenF1Client::new();
    let sessions = client.get_recent_sessions().await?;
    Ok(Json(sessions))
}

/// Handler serving `GET /api/openf1/laps/:session_key/:driver_number`.
#[utoipa::path(
    get,
    path = "/api/openf1/laps/{session_key}/{driver_number}",
    params(
        ("session_key" = u64, Path, description = "OpenF1 Session Key"),
        ("driver_number" = u32, Path, description = "F1 Driver Number (e.g. 1, 44, 16)")
    ),
    responses(
        (status = 200, description = "Historic lap telemetry traces from OpenF1 API", body = Vec<OpenF1Lap>)
    )
)]
pub async fn get_openf1_laps(
    Path((session_key, driver_number)): Path<(u64, u32)>,
) -> Result<Json<Vec<OpenF1Lap>>, AppError> {
    let client = OpenF1Client::new();
    let laps = client.get_laps(session_key, driver_number).await?;
    Ok(Json(laps))
}

/// Request payload for replaying OpenF1 telemetry into a simulation `RaceState`.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct ReplayStateRequest {
    pub session_key: u64,
    pub ego_driver_number: u32,
    pub current_lap: u32,
}

/// Handler serving `POST /api/openf1/replay-state`.
#[utoipa::path(
    post,
    path = "/api/openf1/replay-state",
    request_body = ReplayStateRequest,
    responses(
        (status = 200, description = "Populated simulation RaceState derived from OpenF1 session", body = RaceState)
    )
)]
pub async fn replay_openf1_state(
    Json(payload): Json<ReplayStateRequest>,
) -> Result<Json<RaceState>, AppError> {
    let client = OpenF1Client::new();
    let laps = client
        .get_laps(payload.session_key, payload.ego_driver_number)
        .await
        .unwrap_or_default();

    let current_lap = payload.current_lap;
    let tire_age = laps
        .iter()
        .filter(|l| l.lap_number <= current_lap)
        .count() as u32;

    let compound_str = laps
        .last()
        .and_then(|l| l.compound.as_deref())
        .unwrap_or("MEDIUM");

    let current_tire = match compound_str.to_uppercase().as_str() {
        "SOFT" => TireCompound::Soft,
        "HARD" => TireCompound::Hard,
        "INTERMEDIATE" => TireCompound::Intermediate,
        "WET" => TireCompound::Wet,
        _ => TireCompound::Medium,
    };

    let state = RaceState {
        track: TrackParameters {
            total_laps: 53,
            base_pit_stop_loss_seconds: 22.5,
            historical_sc_probability: 0.35,
        },
        environment: TrackStatus::Green,
        ego_car: EgoCar {
            current_lap,
            current_tire,
            tire_age_laps: tire_age.max(1),
            mandatory_pit_completed: false,
            front_wing_damage: false,
            time_penalty_seconds: 0.0,
        },
        competitors: vec![
            CompetitorCar {
                driver_name: "NOR".to_string(),
                team: "McLaren".to_string(),
                current_lap,
                gap_to_ego_seconds: 2.1,
                current_tire: TireCompound::Medium,
                tire_age_laps: tire_age.max(1),
                projected_pit_lap: current_lap + 10,
            },
            CompetitorCar {
                driver_name: "LEC".to_string(),
                team: "Ferrari".to_string(),
                current_lap,
                gap_to_ego_seconds: -3.5,
                current_tire: TireCompound::Hard,
                tire_age_laps: tire_age.max(1),
                projected_pit_lap: current_lap + 15,
            },
        ],
        weather_forecast: None,
    };

    Ok(Json(state))
}
