//! Vehicle telemetry and race environment state domain models.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::tire::TireCompound;
use super::track::{TrackParameters, TrackStatus, WeatherForecast};

/// State of the primary car for which strategies are optimized.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct EgoCar {
    /// Current lap number completed/in-progress.
    pub current_lap: u32,
    /// Currently fitted tire compound.
    pub current_tire: TireCompound,
    /// Age of current tire set in laps.
    pub tire_age_laps: u32,
    /// Flag indicating whether mandatory dry compound change rule has been satisfied.
    pub mandatory_pit_completed: bool,
    /// Flag indicating front wing aerodynamic damage (+10.0s pit stop penalty).
    pub front_wing_damage: bool,
    /// Time penalty accrued to be served during next pit stop (seconds).
    pub time_penalty_seconds: f64,
}

/// Competitor vehicle state for traffic positioning and dirty air modeling.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct CompetitorCar {
    /// Competitor driver name or code (e.g. `"VER"`).
    pub driver_name: String,
    /// Constructor team name (e.g. `"Red Bull Racing"`).
    pub team: String,
    /// Current lap completed by competitor.
    pub current_lap: u32,
    /// Time gap relative to ego car (positive = ahead of ego car, negative = behind).
    pub gap_to_ego_seconds: f64,
    /// Currently fitted tire compound on competitor car.
    pub current_tire: TireCompound,
    /// Age of competitor's current tires in laps.
    pub tire_age_laps: u32,
    /// Estimated lap on which competitor will execute their next pit stop.
    pub projected_pit_lap: u32,
}

/// Comprehensive snapshot of race conditions for strategy optimization.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct RaceState {
    /// Circuit track configuration parameters.
    pub track: TrackParameters,
    /// Current track flag / safety car condition.
    pub environment: TrackStatus,
    /// Telemetry state of ego car.
    pub ego_car: EgoCar,
    /// Telemetry state of rival competitor cars.
    #[serde(default)]
    pub competitors: Vec<CompetitorCar>,
    /// Live or projected weather forecast transition data.
    #[serde(default)]
    pub weather_forecast: Option<WeatherForecast>,
}
