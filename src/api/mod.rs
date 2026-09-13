pub mod handlers;
pub mod openf1;
pub mod proxy;
pub mod tracks;

use axum::{
    Router,
    routing::{get, post},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::domain::{
    CircuitInfo, CompetitorCar, EgoCar, MonteCarloMetrics, MultiStrategyResponse, PitStopPlan,
    RaceState, StrategyOption, TireCompound, TrackParameters, TrackStatus, TrafficWindowResponse,
    UndercutOvercutOption, WeatherForecast, WeatherState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::simulate_race,
        handlers::get_tracks,
        handlers::get_circuit_geometry,
        tracks::get_circuit_geometry_by_id,
        openf1::get_openf1_sessions,
        openf1::get_openf1_drivers,
        openf1::get_openf1_laps,
        openf1::get_openf1_session_laps,
        openf1::get_openf1_gps_telemetry,
        openf1::get_openf1_locations,
        openf1::get_openf1_positions,
        openf1::get_openf1_intervals,
        openf1::get_openf1_stints,
        openf1::get_openf1_race_control,
        openf1::get_openf1_weather,
        openf1::get_live_state,
        openf1::get_baked_track,
        openf1::replay_openf1_state,
    ),
    components(schemas(
        RaceState,
        TrackParameters,
        TrackStatus,
        EgoCar,
        CompetitorCar,
        TireCompound,
        PitStopPlan,
        StrategyOption,
        MonteCarloMetrics,
        UndercutOvercutOption,
        TrafficWindowResponse,
        MultiStrategyResponse,
        CircuitInfo,
        WeatherState,
        WeatherForecast,
        openf1::OpenF1Session,
        openf1::OpenF1Driver,
        openf1::OpenF1Lap,
        openf1::OpenF1CarData,
        openf1::OpenF1Location,
        openf1::NormalizedTrackPoint,
        openf1::DriverGpsSample,
        openf1::OpenF1Position,
        openf1::OpenF1Interval,
        openf1::OpenF1Stint,
        openf1::OpenF1RaceControl,
        openf1::OpenF1Weather,
        openf1::LiveDriverState,
        openf1::LiveStateResponse,
        openf1::ReplayStateRequest,
    )),
    tags(
        (name = "F1 Strategy Engine", description = "API for simulating multi-stop race strategies & OpenF1 integration")
    )
)]
pub struct ApiDoc;

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(handlers::index_handler))
        .route("/api/tracks", get(handlers::get_tracks))
        .route("/simulate", post(handlers::simulate_race))
        .route("/api/circuit-geometry", get(handlers::get_circuit_geometry))
        .route(
            "/api/circuit-geometry/{track_id}",
            get(tracks::get_circuit_geometry_by_id),
        )
        .route("/api/track/{session_key}", get(openf1::get_baked_track))
        .route("/api/openf1/sessions", get(openf1::get_openf1_sessions))
        .route("/api/openf1/drivers", get(openf1::get_openf1_drivers))
        .route(
            "/api/openf1/laps/{session_key}/{driver_number}",
            get(openf1::get_openf1_laps),
        )
        .route(
            "/api/openf1/session-laps",
            get(openf1::get_openf1_session_laps),
        )
        .route(
            "/api/openf1/gps-telemetry",
            get(openf1::get_openf1_gps_telemetry),
        )
        .route("/api/openf1/location", get(openf1::get_openf1_locations))
        .route("/api/openf1/position", get(openf1::get_openf1_positions))
        .route("/api/openf1/intervals", get(openf1::get_openf1_intervals))
        .route("/api/openf1/stints", get(openf1::get_openf1_stints))
        .route(
            "/api/openf1/race_control",
            get(openf1::get_openf1_race_control),
        )
        .route("/api/openf1/weather", get(openf1::get_openf1_weather))
        .route("/api/live-state/{session_key}", get(openf1::get_live_state))
        .route(
            "/api/openf1/replay-state",
            post(openf1::replay_openf1_state),
        )
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
