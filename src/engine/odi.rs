//! Dynamic Overtaking Difficulty Index (ODI) engine.
//!
//! Evaluates variable, per-lap overtaking difficulty based on:
//! - Track base geometric characteristics (straights vs tight chicanes)
//! - Fuel load burn-off (lighter cars brake later and have more agility)
//! - Tyre compound and age delta (fresh softs vs worn hards)
//! - Thermal dirty air degradation from following within DRS range

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents dynamic, per-lap overtaking difficulty assessment.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct DynamicOdiResult {
    /// Resulting dynamic ODI for the current lap (0.0 = effortless, 1.0 = near impossible).
    pub dynamic_odi: f64,
    /// Baseline geometric track difficulty factor.
    pub base_track_odi: f64,
    /// Fuel burn-off modifier (negative = lighter car eases overtaking).
    pub fuel_modifier: f64,
    /// Tyre pace delta modifier (negative = grip advantage eases overtaking).
    pub tyre_modifier: f64,
    /// Thermal dirty air penalty modifier from consecutive following laps.
    pub dirty_air_modifier: f64,
    /// Qualitative difficulty level string (e.g. "Low", "Moderate", "High", "Extreme").
    pub difficulty_rating: String,
    /// Estimated probability percentage (0-100%) of completing an overtake with a 0.5s pace advantage.
    pub benchmark_overtake_probability_pct: f64,
}

/// Baseline overtaking difficulty rating for standard Formula 1 circuits.
pub fn get_base_track_odi(circuit_id: &str) -> f64 {
    let clean = circuit_id.to_lowercase();
    match clean.as_str() {
        "monaco" | "mc-1929" | "circuit de monaco" => 0.95,
        "budapest" | "hungaroring" | "hu-1986" => 0.82,
        "singapore" | "marina bay" | "sg-2008" => 0.78,
        "zandvoort" | "nl-1948" => 0.75,
        "suzuka" | "jp-1962" => 0.65,
        "barcelona" | "catalunya" | "es-1991" => 0.62,
        "melbourne" | "albert park" | "au-1953" => 0.55,
        "cota" | "austin" | "us-2012" => 0.50,
        "silverstone" | "gb-1948" => 0.42,
        "bahrain" | "sakhir" | "bh-2002" => 0.38,
        "spielberg" | "red bull ring" | "at-1969" => 0.35,
        "spa" | "spa-francorchamps" | "be-1925" => 0.30,
        "baku" | "baku city circuit" | "az-2016" => 0.28,
        "las vegas" | "us-2023" => 0.25,
        "monza" | "it-1922" | "autodromo nazionale monza" => 0.20,
        _ => 0.50, // Default neutral difficulty
    }
}

/// Computes the dynamic per-lap Overtaking Difficulty Index.
///
/// # Arguments
/// * `base_odi` - Base circuit index (0.0 to 1.0).
/// * `current_lap` - Current lap number.
/// * `total_laps` - Total laps in the Grand Prix.
/// * `following_laps_count` - Consecutive laps spent within dirty air (<1.0s behind competitor).
/// * `tyre_delta_seconds` - Net pace delta from tyre advantage (positive = chasing car has fresher/softer tyres).
pub fn calculate_dynamic_odi(
    base_odi: f64,
    current_lap: u32,
    total_laps: u32,
    following_laps_count: u32,
    tyre_delta_seconds: f64,
) -> DynamicOdiResult {
    let total_laps = total_laps.max(1);
    let race_progress = (current_lap as f64 / total_laps as f64).clamp(0.0, 1.0);

    // Fuel effect: Starts with high fuel weight (harder to stop/turn), burns off to low fuel
    // Lighter car braking distances shrink, but driver can take alternate apex lines
    let fuel_modifier = (1.0 - race_progress) * 0.08 - 0.04; // +0.04 at start, -0.04 at finish

    // Tyre delta effect: 1.0s tyre advantage reduces ODI by ~0.12
    let tyre_modifier = -(tyre_delta_seconds * 0.12).clamp(-0.25, 0.25);

    // Thermal dirty air penalty: each consecutive lap within 1.0s overheats surface rubber
    let dirty_air_modifier = (following_laps_count as f64 * 0.03).min(0.18);

    let raw_odi = base_odi + fuel_modifier + tyre_modifier + dirty_air_modifier;
    let dynamic_odi = (raw_odi * 1000.0).round() / 1000.0;
    let dynamic_odi = dynamic_odi.clamp(0.05, 0.98);

    let difficulty_rating = match dynamic_odi {
        d if d < 0.30 => "Low (High Overtaking)".to_string(),
        d if d < 0.55 => "Moderate".to_string(),
        d if d < 0.75 => "High (Difficult)".to_string(),
        _ => "Extreme (Monaco-like)".to_string(),
    };

    // Calculate benchmark probability with 0.5s pace advantage
    let benchmark_prob = calculate_overtake_probability(dynamic_odi, 0.5);

    DynamicOdiResult {
        dynamic_odi,
        base_track_odi: base_odi,
        fuel_modifier: (fuel_modifier * 1000.0).round() / 1000.0,
        tyre_modifier: (tyre_modifier * 1000.0).round() / 1000.0,
        dirty_air_modifier: (dirty_air_modifier * 1000.0).round() / 1000.0,
        difficulty_rating,
        benchmark_overtake_probability_pct: (benchmark_prob * 100.0).round(),
    }
}

/// Estimates on-track overtake probability (0.0 to 1.0) given current dynamic ODI
/// and the attacking car's overall pace delta in seconds (positive = attacking car is faster).
pub fn calculate_overtake_probability(dynamic_odi: f64, pace_delta_seconds: f64) -> f64 {
    if pace_delta_seconds <= -0.5 {
        return 0.0; // Slower car cannot overtake without mistakes
    }

    // Required pace advantage threshold to overcome dirty air at this circuit
    let delta_threshold = 0.3 + (dynamic_odi * 1.5); // Monaco: needs ~1.7s delta; Monza: needs ~0.6s

    let effective_surplus = pace_delta_seconds - delta_threshold;
    if effective_surplus <= 0.0 {
        // Slim chance on competitor error
        (0.05 * (1.0 - dynamic_odi)).clamp(0.01, 0.15)
    } else {
        // Sigmoid curve mapping excess delta to overtake probability
        let prob = 0.15 + (1.0 - (-effective_surplus * 1.8).exp()) * 0.85;
        (prob * (1.0 - dynamic_odi * 0.3)).clamp(0.05, 0.98)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monaco_vs_monza_odi() {
        let monaco_base = get_base_track_odi("monaco");
        let monza_base = get_base_track_odi("monza");
        assert!(monaco_base > 0.90);
        assert!(monza_base < 0.30);
    }

    #[test]
    fn test_dynamic_odi_fuel_burn() {
        let early = calculate_dynamic_odi(0.50, 1, 50, 0, 0.0);
        let late = calculate_dynamic_odi(0.50, 48, 50, 0, 0.0);
        // Late race with low fuel should be easier (lower ODI) than early race full fuel
        assert!(late.dynamic_odi < early.dynamic_odi);
    }

    #[test]
    fn test_dynamic_odi_tyre_delta() {
        let fresh_tyres = calculate_dynamic_odi(0.50, 25, 50, 0, 1.5);
        let worn_tyres = calculate_dynamic_odi(0.50, 25, 50, 0, -1.5);
        assert!(fresh_tyres.dynamic_odi < worn_tyres.dynamic_odi);
    }

    #[test]
    fn test_dynamic_odi_dirty_air_overheating() {
        let lap_1_behind = calculate_dynamic_odi(0.50, 25, 50, 1, 0.0);
        let lap_5_behind = calculate_dynamic_odi(0.50, 25, 50, 5, 0.0);
        assert!(lap_5_behind.dynamic_odi > lap_1_behind.dynamic_odi);
    }
}
