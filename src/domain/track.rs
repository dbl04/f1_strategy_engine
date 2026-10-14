//! Track state, circuit metadata, and weather condition domain models.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Track flag status and neutralization state.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub enum TrackStatus {
    /// Green flag racing conditions (full race pace).
    #[serde(alias = "GREEN", alias = "green")]
    Green,
    /// Localized yellow flag warning.
    #[serde(alias = "YELLOW", alias = "yellow")]
    Yellow,
    /// Full Safety Car deployed (~40% pit stop time loss discount).
    #[serde(alias = "SAFETY_CAR", alias = "safety_car", alias = "SC", alias = "SafetyCar")]
    SafetyCar,
    /// Virtual Safety Car deployed (~30% pit stop time loss discount).
    #[serde(alias = "VIRTUAL_SAFETY_CAR", alias = "virtual_safety_car", alias = "VSC", alias = "VirtualSafetyCar")]
    VirtualSafetyCar,
    /// Red flag race session suspended.
    #[serde(alias = "RED_FLAG", alias = "red_flag", alias = "RED", alias = "RedFlag")]
    RedFlag,
}

/// Weather state on track surface.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub enum WeatherState {
    /// Dry track surface (slick tires required).
    #[serde(alias = "DRY", alias = "dry")]
    Dry,
    /// Damp track surface (Intermediate tire crossover window).
    #[serde(alias = "DAMP", alias = "damp")]
    Damp,
    /// Wet track surface (Full Wet tire crossover window).
    #[serde(alias = "WET", alias = "wet")]
    Wet,
}

/// Weather forecast transition specification.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct WeatherForecast {
    /// Initial weather condition at current lap.
    pub current_weather: WeatherState,
    /// Projected weather condition following transition lap.
    pub predicted_weather: WeatherState,
    /// Lap number at which weather transition occurs.
    pub transition_lap: u32,
}

/// Track physical configuration parameters.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct TrackParameters {
    /// Total number of laps in the Grand Prix.
    #[serde(default = "default_total_laps")]
    pub total_laps: u32,
    /// Baseline stationary + pit lane speed limiter time loss (seconds).
    #[serde(alias = "base_pit_loss", alias = "pit_loss_seconds", default = "default_pit_loss")]
    pub base_pit_stop_loss_seconds: f64,
    /// Historical Safety Car deployment probability factor for circuit (0.0 to 1.0).
    #[serde(default)]
    pub historical_sc_probability: f64,
}

fn default_total_laps() -> u32 {
    53
}

fn default_pit_loss() -> f64 {
    22.5
}

/// Static circuit metadata and telemetry descriptor.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct CircuitInfo {
    /// Unique identifier string (e.g. `"monza"`).
    pub id: String,
    /// Official circuit name (e.g. `"Autodromo Nazionale Monza"`).
    pub name: String,
    /// Host country name.
    pub country: String,
    /// Flag emoji representation.
    pub flag_emoji: String,
    /// Total Grand Prix race distance in laps.
    pub total_laps: u32,
    /// Standard pit lane time loss penalty in seconds under Green Flag conditions.
    pub base_pit_stop_loss_seconds: f64,
    /// Lap distance length in kilometers.
    pub length_km: f64,
    /// OpenStreetMap query name parameter for vector geometry fetching.
    pub osm_query_name: String,
    /// Historical Safety Car likelihood rating.
    pub historical_sc_probability: f64,
}
