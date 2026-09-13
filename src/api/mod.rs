pub mod handlers;
pub mod openf1;
pub mod proxy;
pub mod tracks;

use axum::{
    routing::{get, post},
    Router,
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
        openf1::get_openf1_laps,
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
        openf1::OpenF1Lap,
        openf1::OpenF1CarData,
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
        .route("/api/circuit-geometry/{track_id}", get(tracks::get_circuit_geometry_by_id))
        .route("/api/openf1/sessions", get(openf1::get_openf1_sessions))
        .route("/api/openf1/laps/{session_key}/{driver_number}", get(openf1::get_openf1_laps))
        .route("/api/openf1/replay-state", post(openf1::replay_openf1_state))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
}

