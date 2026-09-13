//! FIA sporting regulations compliance module.

use crate::domain::{EgoCar, TireCompound};

/// Validates whether a strategy sequence complies with mandatory FIA Formula 1 sporting rules.
///
/// In dry races, drivers must use at least two different dry tire compounds unless a mandatory pit stop
/// has already been completed or wet weather tires are deployed.
///
/// # Arguments
/// * `ego_car` - Telemetry state of ego car.
/// * `new_compounds` - Slice of candidate new tire compounds selected for future pit stops.
///
/// # Returns
/// `true` if the strategy satisfies FIA compound rules, `false` otherwise.
pub fn is_valid_f1_strategy(ego_car: &EgoCar, new_compounds: &[TireCompound]) -> bool {
    if ego_car.mandatory_pit_completed {
        return true;
    }

    for compound in new_compounds {
        if *compound != ego_car.current_tire {
            return true;
        }
    }

    false
}

