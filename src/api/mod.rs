pub mod handlers;
pub mod proxy;

use axum::{
    routing::{get, post},
    Router,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::domain::{
    CircuitInfo, CompetitorCar, EgoCar, MultiStrategyResponse, PitStopPlan, RaceState,
    StrategyOption, TireCompound, TrackParameters, TrackStatus, TrafficWindowResponse,
    UndercutOvercutOption, WeatherForecast, WeatherState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::simulate_race,
        handlers::get_tracks,
        handlers::get_circuit_geometry
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
        UndercutOvercutOption,
        TrafficWindowResponse,
        MultiStrategyResponse,
        CircuitInfo,
        WeatherState,
        WeatherForecast
    )),
    tags(
        (name = "F1 Strategy Engine", description = "API for simulating multi-stop race strategies")
    )
)]
pub struct ApiDoc;

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(handlers::index_handler))
        .route("/api/tracks", get(handlers::get_tracks))
        .route("/simulate", post(handlers::simulate_race))
        .route("/api/circuit-geometry", get(handlers::get_circuit_geometry))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
}
