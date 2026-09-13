//! Tire compound definitions and lifecycle models.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Formula 1 tire compound choices.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
pub enum TireCompound {
    /// Soft compound (fastest base pace, highest degradation rate).
    Soft,
    /// Medium compound (balanced pace and durability).
    Medium,
    /// Hard compound (slowest base pace, lowest degradation rate).
    Hard,
    /// Intermediate compound (light water / damp track conditions).
    Intermediate,
    /// Wet compound (full wet track conditions).
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

