//! Axum route handlers exposing OpenF1 telemetry and session endpoints.

use axum::{
    Json,
    extract::{Path, Query},
};

use crate::domain::{CompetitorCar, EgoCar, RaceState, TireCompound, TrackParameters, TrackStatus};
use crate::error::AppError;

use super::client::OpenF1Client;
use super::models::{
    DriverGpsSample, LiveStateResponse, LocationQueryParams, NormalizedTrackPoint, OpenF1Driver,
    OpenF1Interval, OpenF1Lap, OpenF1Location, OpenF1Position, OpenF1RaceControl, OpenF1Session,
    OpenF1Stint, OpenF1Weather, ReplayStateRequest, SessionOnlyQueryParams, SessionQueryParams,
};

/// Handler serving `GET /api/openf1/sessions?year=2026`.
#[utoipa::path(
    get,
    path = "/api/openf1/sessions",
    params(SessionQueryParams),
    responses(
        (status = 200, description = "List of F1 sessions from OpenF1 API", body = Vec<OpenF1Session>)
    )
)]
pub async fn get_openf1_sessions(
    Query(params): Query<SessionQueryParams>,
) -> Result<Json<Vec<OpenF1Session>>, AppError> {
    let client = OpenF1Client::new();
    let year = params.year.unwrap_or(2026);
    let sessions = client.get_sessions(year).await?;
    Ok(Json(sessions))
}

/// Handler serving `GET /api/openf1/drivers?session_key=...`.
#[utoipa::path(
    get,
    path = "/api/openf1/drivers",
    params(SessionOnlyQueryParams),
    responses(
        (status = 200, description = "List of driver participants in session from OpenF1 API", body = Vec<OpenF1Driver>)
    )
)]
pub async fn get_openf1_drivers(
    Query(params): Query<SessionOnlyQueryParams>,
) -> Result<Json<Vec<OpenF1Driver>>, AppError> {
    let client = OpenF1Client::new();
    let drivers = client.get_drivers(params.session_key).await?;
    Ok(Json(drivers))
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

/// Handler serving `GET /api/openf1/session-laps?session_key=...`.
#[utoipa::path(
    get,
    path = "/api/openf1/session-laps",
    params(SessionOnlyQueryParams),
    responses(
        (status = 200, description = "All lap records for a session from OpenF1 API", body = Vec<OpenF1Lap>)
    )
)]
pub async fn get_openf1_session_laps(
    Query(params): Query<SessionOnlyQueryParams>,
) -> Result<Json<Vec<OpenF1Lap>>, AppError> {
    let client = OpenF1Client::new();
    let laps = client.get_all_session_laps(params.session_key).await?;
    Ok(Json(laps))
}

/// Handler serving `GET /api/openf1/gps-telemetry?session_key=...`.
#[utoipa::path(
    get,
    path = "/api/openf1/gps-telemetry",
    params(SessionOnlyQueryParams),
    responses(
        (status = 200, description = "Downsampled high-precision GPS coordinate stream for all drivers in session", body = Vec<DriverGpsSample>)
    )
)]
pub async fn get_openf1_gps_telemetry(
    Query(params): Query<SessionOnlyQueryParams>,
) -> Result<Json<Vec<DriverGpsSample>>, AppError> {
    let client = OpenF1Client::new();
    let samples = client.get_session_gps_telemetry(params.session_key).await?;
    Ok(Json(samples))
}

/// Handler serving `GET /api/openf1/location?session_key=...`.
#[utoipa::path(
    get,
    path = "/api/openf1/location",
    params(LocationQueryParams),
    responses(
        (status = 200, description = "Live 3D location telemetry samples for drivers in session", body = Vec<OpenF1Location>)
    )
)]
pub async fn get_openf1_locations(
    Query(params): Query<LocationQueryParams>,
) -> Result<Json<Vec<OpenF1Location>>, AppError> {
    let client = OpenF1Client::new();
    let locations = client
        .get_locations(params.session_key, params.date_gt.as_deref())
        .await?;
    Ok(Json(locations))
}

/// Handler serving `GET /api/openf1/position?session_key=...`.
#[utoipa::path(
    get,
    path = "/api/openf1/position",
    params(SessionOnlyQueryParams),
    responses(
        (status = 200, description = "Current position standing table for session", body = Vec<OpenF1Position>)
    )
)]
pub async fn get_openf1_positions(
    Query(params): Query<SessionOnlyQueryParams>,
) -> Result<Json<Vec<OpenF1Position>>, AppError> {
    let client = OpenF1Client::new();
    let positions = client.get_positions(params.session_key).await?;
    Ok(Json(positions))
}

/// Handler serving `GET /api/openf1/intervals?session_key=...`.
#[utoipa::path(
    get,
    path = "/api/openf1/intervals",
    params(SessionOnlyQueryParams),
    responses(
        (status = 200, description = "Current driver gap & interval telemetry for session", body = Vec<OpenF1Interval>)
    )
)]
pub async fn get_openf1_intervals(
    Query(params): Query<SessionOnlyQueryParams>,
) -> Result<Json<Vec<OpenF1Interval>>, AppError> {
    let client = OpenF1Client::new();
    let intervals = client.get_intervals(params.session_key).await?;
    Ok(Json(intervals))
}

/// Handler serving `GET /api/openf1/stints?session_key=...`.
#[utoipa::path(
    get,
    path = "/api/openf1/stints",
    params(SessionOnlyQueryParams),
    responses(
        (status = 200, description = "Driver tyre stint telemetry for session", body = Vec<OpenF1Stint>)
    )
)]
pub async fn get_openf1_stints(
    Query(params): Query<SessionOnlyQueryParams>,
) -> Result<Json<Vec<OpenF1Stint>>, AppError> {
    let client = OpenF1Client::new();
    let stints = client.get_stints(params.session_key).await?;
    Ok(Json(stints))
}

/// Handler serving `GET /api/openf1/race_control?session_key=...`.
#[utoipa::path(
    get,
    path = "/api/openf1/race_control",
    params(SessionOnlyQueryParams),
    responses(
        (status = 200, description = "Race control event messages and flag status", body = Vec<OpenF1RaceControl>)
    )
)]
pub async fn get_openf1_race_control(
    Query(params): Query<SessionOnlyQueryParams>,
) -> Result<Json<Vec<OpenF1RaceControl>>, AppError> {
    let client = OpenF1Client::new();
    let rc = client.get_race_control(params.session_key).await?;
    Ok(Json(rc))
}

/// Handler serving `GET /api/openf1/weather?session_key=...`.
#[utoipa::path(
    get,
    path = "/api/openf1/weather",
    params(SessionOnlyQueryParams),
    responses(
        (status = 200, description = "Session ambient & track weather metrics", body = Vec<OpenF1Weather>)
    )
)]
pub async fn get_openf1_weather(
    Query(params): Query<SessionOnlyQueryParams>,
) -> Result<Json<Vec<OpenF1Weather>>, AppError> {
    let client = OpenF1Client::new();
    let weather = client.get_weather(params.session_key).await?;
    Ok(Json(weather))
}

/// Handler serving `GET /api/live-state/:session_key`.
#[utoipa::path(
    get,
    path = "/api/live-state/{session_key}",
    params(
        ("session_key" = u64, Path, description = "OpenF1 Session Key")
    ),
    responses(
        (status = 200, description = "Rate-limit shielded live state aggregation payload", body = LiveStateResponse)
    )
)]
pub async fn get_live_state(
    Path(session_key): Path<u64>,
) -> Result<Json<LiveStateResponse>, AppError> {
    let client = OpenF1Client::new();
    let state = client.get_live_state(session_key).await?;
    Ok(Json(state))
}

/// Handler serving `GET /api/track/:session_key`.
#[utoipa::path(
    get,
    path = "/api/track/{session_key}",
    params(
        ("session_key" = u64, Path, description = "OpenF1 Session Key")
    ),
    responses(
        (status = 200, description = "Baked normalized track geometry points (0.0 - 1.0 scale)", body = Vec<NormalizedTrackPoint>)
    )
)]
pub async fn get_baked_track(
    Path(session_key): Path<u64>,
) -> Result<Json<Vec<NormalizedTrackPoint>>, AppError> {
    let client = OpenF1Client::new();
    let track = client.get_baked_track(session_key).await?;
    Ok(Json(track))
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
    let tire_age = laps.iter().filter(|l| l.lap_number <= current_lap).count() as u32;

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
