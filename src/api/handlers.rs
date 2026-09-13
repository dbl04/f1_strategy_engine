//! HTTP Request handlers for the F1 Strategy Engine endpoints.

use axum::{Json, extract::Query, response::Html};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::api::proxy::fetch_circuit_geometry;
use crate::domain::{CircuitInfo, MultiStrategyResponse, RaceState};
use crate::engine::optimizer::optimize_race_strategies;
use crate::error::AppError;

/// Serves the interactive F1 TV Broadcast Web Dashboard HTML interface at `GET /`.
pub async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../assets/index.html"))
}

/// Retrieves metadata for all supported official F1 circuits.
#[utoipa::path(
    get,
    path = "/api/tracks",
    responses(
        (status = 200, description = "List of official F1 circuits", body = Vec<CircuitInfo>)
    )
)]
pub async fn get_tracks() -> Json<Vec<CircuitInfo>> {
    let tracks = vec![
        CircuitInfo {
            id: "monza".to_string(),
            name: "Autodromo Nazionale Monza".to_string(),
            country: "Italy".to_string(),
            flag_emoji: "🇮🇹".to_string(),
            total_laps: 53,
            base_pit_stop_loss_seconds: 25.0,
            length_km: 5.793,
            osm_query_name: "Autodromo Nazionale Monza".to_string(),
            historical_sc_probability: 0.25,
        },
        CircuitInfo {
            id: "silverstone".to_string(),
            name: "Silverstone Circuit".to_string(),
            country: "UK".to_string(),
            flag_emoji: "🇬🇧".to_string(),
            total_laps: 52,
            base_pit_stop_loss_seconds: 20.5,
            length_km: 5.891,
            osm_query_name: "Silverstone Circuit".to_string(),
            historical_sc_probability: 0.30,
        },
        CircuitInfo {
            id: "spa".to_string(),
            name: "Circuit de Spa-Francorchamps".to_string(),
            country: "Belgium".to_string(),
            flag_emoji: "🇧🇪".to_string(),
            total_laps: 44,
            base_pit_stop_loss_seconds: 22.0,
            length_km: 7.004,
            osm_query_name: "Circuit de Spa-Francorchamps".to_string(),
            historical_sc_probability: 0.50,
        },
        CircuitInfo {
            id: "monaco".to_string(),
            name: "Circuit de Monaco".to_string(),
            country: "Monaco".to_string(),
            flag_emoji: "🇲🇨".to_string(),
            total_laps: 78,
            base_pit_stop_loss_seconds: 19.5,
            length_km: 3.337,
            osm_query_name: "Circuit de Monaco".to_string(),
            historical_sc_probability: 0.80,
        },
        CircuitInfo {
            id: "bahrain".to_string(),
            name: "Bahrain International Circuit".to_string(),
            country: "Bahrain".to_string(),
            flag_emoji: "🇧🇭".to_string(),
            total_laps: 57,
            base_pit_stop_loss_seconds: 22.5,
            length_km: 5.412,
            osm_query_name: "Bahrain International Circuit".to_string(),
            historical_sc_probability: 0.35,
        },
        CircuitInfo {
            id: "suzuka".to_string(),
            name: "Suzuka International Racing Course".to_string(),
            country: "Japan".to_string(),
            flag_emoji: "🇯🇵".to_string(),
            total_laps: 53,
            base_pit_stop_loss_seconds: 22.5,
            length_km: 5.807,
            osm_query_name: "Suzuka Circuit".to_string(),
            historical_sc_probability: 0.40,
        },
        CircuitInfo {
            id: "cota".to_string(),
            name: "Circuit of the Americas".to_string(),
            country: "USA".to_string(),
            flag_emoji: "🇺🇸".to_string(),
            total_laps: 56,
            base_pit_stop_loss_seconds: 20.0,
            length_km: 5.513,
            osm_query_name: "Circuit of the Americas".to_string(),
            historical_sc_probability: 0.30,
        },
        CircuitInfo {
            id: "singapore".to_string(),
            name: "Marina Bay Street Circuit".to_string(),
            country: "Singapore".to_string(),
            flag_emoji: "🇸🇬".to_string(),
            total_laps: 62,
            base_pit_stop_loss_seconds: 28.0,
            length_km: 4.940,
            osm_query_name: "Marina Bay Street Circuit".to_string(),
            historical_sc_probability: 1.0,
        },
    ];

    Json(tracks)
}

/// Query parameters for fetching circuit geometry.
#[derive(Deserialize, IntoParams)]
pub struct OverpassQuery {
    /// Circuit name query string (e.g. `"Autodromo Nazionale Monza"`).
    pub name: String,
}

/// Proxies OpenStreetMap Overpass API circuit geometry to bypass browser CORS constraints.
#[utoipa::path(
    get,
    path = "/api/circuit-geometry",
    params(OverpassQuery),
    responses(
        (status = 200, description = "Raw Overpass API JSON response with circuit geometry")
    )
)]
pub async fn get_circuit_geometry(
    Query(params): Query<OverpassQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let json = fetch_circuit_geometry(&params.name).await?;
    Ok(Json(json))
}

/// Executes strategy optimization calculations for a given input race state.
#[utoipa::path(
    post,
    path = "/simulate",
    request_body = RaceState,
    responses(
        (status = 200, description = "Multi-strategy simulation result", body = MultiStrategyResponse)
    )
)]
pub async fn simulate_race(Json(payload): Json<RaceState>) -> Json<MultiStrategyResponse> {
    let response = optimize_race_strategies(&payload);
    Json(response)
}
