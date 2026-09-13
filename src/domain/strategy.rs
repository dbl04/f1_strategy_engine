//! Pit strategy outputs, undercut/overcut calculations, and traffic window domain models.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::tire::TireCompound;

/// Details of an individual planned pit stop.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct PitStopPlan {
    /// Scheduled lap for executing pit stop.
    pub pit_lap: u32,
    /// Target tire compound to fit during stop.
    pub new_compound: TireCompound,
    /// Calculated pit lane stationary & transit time loss (seconds).
    pub pit_loss_seconds: f64,
    /// Human-readable reasons for pit loss adjustments (SC discount, wing repair, penalties).
    pub reasons: Vec<String>,
}

/// A complete race strategy option containing stint breakdowns and time predictions.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct StrategyOption {
    /// Display name of strategy (e.g. `"1-Stop: Medium (L20) -> Hard"`).
    pub name: String,
    /// Flag indicating compliance with FIA sporting rules (mandatory compound changes).
    pub is_valid_f1_rules: bool,
    /// Total number of pit stops in strategy sequence.
    pub total_pit_stops: u32,
    /// Sequential list of planned pit stops.
    pub pit_stops: Vec<PitStopPlan>,
    /// Projected total race duration from current lap to finish in seconds.
    pub projected_total_time_seconds: f64,
    /// Human-readable formatted race duration string (e.g. `"1h 24m 12.5s"`).
    pub projected_total_time_formatted: String,
    /// Calculated average lap time across remaining laps (seconds).
    pub average_lap_time_seconds: f64,
    /// Cumulative lap time lost to tire wear/degradation (seconds).
    pub total_degradation_loss_seconds: f64,
    /// Total cumulative time spent in pit lane across all stops (seconds).
    pub total_pit_stop_time_loss_seconds: f64,
    /// Time delta difference relative to optimal strategy (seconds).
    pub delta_to_optimal_seconds: f64,
}

/// Delta advantage option computed for pitting earlier (undercut) or later (overcut) vs a competitor.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct UndercutOvercutOption {
    /// Competitor driver name or code being targeted.
    pub competitor_name: String,
    /// Competitor's projected pit stop lap.
    pub target_pit_lap: u32,
    /// Lap offset relative to target competitor pit lap (-3 to +3 laps).
    pub pit_lap_delta: i32,
    /// Calculated ego pit stop lap.
    pub ego_pit_lap: u32,
    /// Net time advantage gained in seconds (positive = faster than baseline).
    pub advantage_seconds: f64,
}

/// Pit exit traffic assessment for a candidate pit lap.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct TrafficWindowResponse {
    /// Candidate pit lap evaluated.
    pub pit_lap: u32,
    /// True if ego car will emerge into clean air without immediate traffic.
    pub is_clean_air: bool,
    /// Warning message detailing DRS train risk or traffic proximity upon pit release.
    pub traffic_warning: String,
}

/// Complete strategy engine response payload returned by `/simulate`.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct MultiStrategyResponse {
    /// Fastest valid strategy option identified.
    pub optimal_strategy: StrategyOption,
    /// Alternative strategy options ranked by performance.
    pub alternative_strategies: Vec<StrategyOption>,
    /// True if active Safety Car or VSC provides cheap pit stop advantage.
    pub safety_car_advantage_active: bool,
    /// Executive summary message detailing recommendations.
    pub summary_message: String,
    /// Matrix of undercut/overcut opportunities against competitors.
    #[serde(default)]
    pub undercut_overcut_options: Vec<UndercutOvercutOption>,
    #[serde(default)]
    /// Traffic window analysis for upcoming pit laps.
    pub traffic_windows: Vec<TrafficWindowResponse>,
    /// Calculated per-lap probability of Safety Car deployment based on track history.
    #[serde(default)]
    pub safety_car_probability_per_lap: f64,
}


