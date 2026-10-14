//! Tire compound definitions and lifecycle models.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Formula 1 tire compound choices.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub enum TireCompound {
    /// Soft compound (fastest base pace, highest degradation rate).
    #[serde(alias = "SOFT", alias = "soft")]
    Soft,
    /// Medium compound (balanced pace and durability).
    #[serde(alias = "MEDIUM", alias = "medium")]
    Medium,
    /// Hard compound (slowest base pace, lowest degradation rate).
    #[serde(alias = "HARD", alias = "hard")]
    Hard,
    /// Intermediate compound (light water / damp track conditions).
    #[serde(alias = "INTERMEDIATE", alias = "intermediate", alias = "INTER", alias = "inter")]
    Intermediate,
    /// Wet compound (full wet track conditions).
    #[serde(alias = "WET", alias = "wet")]
    Wet,
}

impl TireCompound {
    /// Returns the baseline lifespan (in laps) before encountering severe non-linear degradation ("tire cliff").
    pub fn lifecycle_laps(&self) -> u32 {
        match self {
            TireCompound::Soft => 20,
            TireCompound::Medium => 35,
            TireCompound::Hard => 50,
            TireCompound::Intermediate => 25,
            TireCompound::Wet => 40,
        }
    }
}
