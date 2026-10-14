//! Driver reference pace estimation and teammate performance blending.
//!
//! Formula 1 strategy requires projecting future lap times based on realistic pace.
//! This module blends a driver's observed pace during the current race with their
//! teammate's performance to capture both individual driver form and team car performance.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Result of driver pace estimation blending individual race laps with teammate reference.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct BlendedPaceResult {
    /// Final blended representative pace in seconds per lap.
    pub blended_pace_seconds: f64,
    /// Driver's own average green-flag lap time in seconds (if available).
    pub driver_own_pace_seconds: Option<f64>,
    /// Teammate's average green-flag lap time in seconds (if available).
    pub teammate_pace_seconds: Option<f64>,
    /// Weight assigned to driver's own pace (e.g. 0.70).
    pub driver_weight: f64,
    /// Weight assigned to teammate's pace (e.g. 0.30).
    pub teammate_weight: f64,
    /// Number of representative laps analyzed for the driver.
    pub driver_laps_analyzed: usize,
    /// Number of representative laps analyzed for the teammate.
    pub teammate_laps_analyzed: usize,
}

/// Computes a trimmed mean of lap times, ignoring extreme outliers (pit-in/out, SC deltas).
fn clean_lap_times(laps: &[f64], baseline: f64) -> Vec<f64> {
    laps.iter()
        .copied()
        .filter(|&t| t >= baseline * 0.90 && t <= baseline * 1.15)
        .collect()
}

/// Blends a driver's observed race pace with their teammate's pace.
///
/// If both drivers have clean laps, uses a 70% driver / 30% teammate weighted blend.
/// If only one driver has laps, uses 100% of available data.
/// If neither has valid laps, falls back to the baseline track pace.
pub fn estimate_blended_pace(
    driver_laps: &[f64],
    teammate_laps: &[f64],
    baseline_lap_time: f64,
) -> BlendedPaceResult {
    let clean_driver = clean_lap_times(driver_laps, baseline_lap_time);
    let clean_teammate = clean_lap_times(teammate_laps, baseline_lap_time);

    let driver_avg = if !clean_driver.is_empty() {
        Some(clean_driver.iter().sum::<f64>() / clean_driver.len() as f64)
    } else {
        None
    };

    let teammate_avg = if !clean_teammate.is_empty() {
        Some(clean_teammate.iter().sum::<f64>() / clean_teammate.len() as f64)
    } else {
        None
    };

    let (blended, d_weight, t_weight) = match (driver_avg, teammate_avg) {
        (Some(d), Some(t)) => {
            // 70% driver's own demonstrated pace + 30% teammate's car-capability benchmark
            let blended = (d * 0.70) + (t * 0.30);
            (blended, 0.70, 0.30)
        }
        (Some(d), None) => (d, 1.0, 0.0),
        (None, Some(t)) => (t, 0.0, 1.0),
        (None, None) => (baseline_lap_time, 0.0, 0.0),
    };

    BlendedPaceResult {
        blended_pace_seconds: (blended * 1000.0).round() / 1000.0,
        driver_own_pace_seconds: driver_avg.map(|p| (p * 1000.0).round() / 1000.0),
        teammate_pace_seconds: teammate_avg.map(|p| (p * 1000.0).round() / 1000.0),
        driver_weight: d_weight,
        teammate_weight: t_weight,
        driver_laps_analyzed: clean_driver.len(),
        teammate_laps_analyzed: clean_teammate.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blended_pace_calculation() {
        let driver_laps = vec![89.0, 89.2, 89.1, 88.9]; // Avg = 89.05
        let teammate_laps = vec![90.0, 90.2, 90.1]; // Avg = 90.10
        let baseline = 90.0;

        let res = estimate_blended_pace(&driver_laps, &teammate_laps, baseline);
        assert_eq!(res.driver_weight, 0.70);
        assert_eq!(res.teammate_weight, 0.30);
        // Blended should be between 89.05 and 90.10, closer to 89.05
        assert!(res.blended_pace_seconds > 89.05 && res.blended_pace_seconds < 90.10);
    }

    #[test]
    fn test_filter_pit_stop_laps() {
        let driver_laps = vec![89.0, 115.0, 89.2]; // 115s was in/out lap
        let baseline = 90.0;

        let res = estimate_blended_pace(&driver_laps, &[], baseline);
        assert_eq!(res.driver_laps_analyzed, 2);
        assert!(res.blended_pace_seconds < 90.0);
    }
}
