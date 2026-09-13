//! Rayon-parallelized stochastic Monte Carlo strategy simulation engine.

use crate::domain::{MonteCarloMetrics, RaceState, StrategyOption, TrackStatus};
use rand::Rng;
use rayon::prelude::*;

/// Executes parallel stochastic Monte Carlo simulations (5,000 iterations by default) for a strategy option.
///
/// In each iteration, stochastic variables are sampled:
/// - Tire degradation rate jitter: uniform variation +/- 5%.
/// - Pit stop execution variance: base pit loss + 5% probability of a +2.0s slow pit stop hazard.
/// - Random Safety Car deployment chance per lap based on historical probability.
///
/// # Arguments
/// * `state` - Current race snapshot state.
/// * `strategy` - Strategy option sequence to evaluate.
/// * `iterations` - Number of Monte Carlo iterations (e.g. 5,000).
///
/// # Returns
/// A [`MonteCarloMetrics`] struct containing expected mean time, P10 ceiling, P90 floor, win probability %, and risk score.
pub fn run_monte_carlo_simulation(
    state: &RaceState,
    strategy: &StrategyOption,
    iterations: usize,
) -> MonteCarloMetrics {
    if iterations == 0 {
        return MonteCarloMetrics {
            expected_time_seconds: strategy.projected_total_time_seconds,
            p10_time_seconds: strategy.projected_total_time_seconds * 0.98,
            p90_time_seconds: strategy.projected_total_time_seconds * 1.02,
            win_probability_pct: if strategy.delta_to_optimal_seconds == 0.0 {
                95.0
            } else {
                20.0
            },
            risk_score: strategy.projected_total_time_seconds * 0.04,
        };
    }

    let current_lap = state.ego_car.current_lap;
    let total_laps = state.track.total_laps;

    let times: Vec<f64> = (0..iterations)
        .into_par_iter()
        .map(|_| {
            let mut rng = rand::thread_rng();

            // Degradation multiplier jitter (+/- 5%)
            let deg_jitter: f64 = rng.gen_range(0.95..=1.05);

            // Pit stop hazard (+2.0s slow stop with 5% probability per stop)
            let mut pit_hazard_total = 0.0;
            for _ in &strategy.pit_stops {
                if rng.gen_bool(0.05) {
                    pit_hazard_total += 2.0;
                }
            }

            // Stochastic SC trigger check across remaining laps
            let sc_prob_per_lap = if total_laps > 0 {
                state.track.historical_sc_probability / (total_laps as f64)
            } else {
                0.0
            };

            let mut sim_env = state.environment.clone();
            let mut sc_discount = 0.0;
            for _ in current_lap..total_laps {
                if sim_env == TrackStatus::Green && rng.gen_bool(sc_prob_per_lap.min(1.0)) {
                    sim_env = TrackStatus::SafetyCar;
                    sc_discount += 5.0;
                }
            }

            let base_time = strategy.projected_total_time_seconds;
            let jitted_time = base_time * deg_jitter + pit_hazard_total - sc_discount;
            jitted_time.max(1.0)
        })
        .collect();

    let mut sorted_times = times;
    sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let len = sorted_times.len();
    let sum: f64 = sorted_times.iter().sum();
    let mean = sum / (len as f64);

    let p10_idx = ((len as f64) * 0.10) as usize;
    let p90_idx = ((len as f64) * 0.90) as usize;

    let p10 = sorted_times[p10_idx.min(len - 1)];
    let p90 = sorted_times[p90_idx.min(len - 1)];
    let risk_score = p90 - p10;

    let win_pct = if strategy.delta_to_optimal_seconds == 0.0 {
        (85.0 + ((mean - p10) * 0.1)).clamp(50.0, 98.0)
    } else {
        (50.0 - (strategy.delta_to_optimal_seconds * 2.0)).max(2.0)
    };

    MonteCarloMetrics {
        expected_time_seconds: (mean * 100.0).round() / 100.0,
        p10_time_seconds: (p10 * 100.0).round() / 100.0,
        p90_time_seconds: (p90 * 100.0).round() / 100.0,
        win_probability_pct: (win_pct * 10.0).round() / 10.0,
        risk_score: (risk_score * 100.0).round() / 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EgoCar, TrackParameters};

    #[test]
    fn test_run_monte_carlo_simulation() {
        let state = RaceState {
            track: TrackParameters {
                total_laps: 50,
                base_pit_stop_loss_seconds: 20.0,
                historical_sc_probability: 0.3,
            },
            environment: TrackStatus::Green,
            ego_car: EgoCar {
                current_lap: 10,
                current_tire: crate::domain::TireCompound::Medium,
                tire_age_laps: 10,
                mandatory_pit_completed: false,
                front_wing_damage: false,
                time_penalty_seconds: 0.0,
            },
            competitors: vec![],
            weather_forecast: None,
        };

        let dummy_strat = StrategyOption {
            name: "1-Stop".to_string(),
            is_valid_f1_rules: true,
            total_pit_stops: 1,
            pit_stops: vec![],
            projected_total_time_seconds: 4500.0,
            projected_total_time_formatted: "1h 15m 0.0s".to_string(),
            average_lap_time_seconds: 90.0,
            total_degradation_loss_seconds: 20.0,
            total_pit_stop_time_loss_seconds: 20.0,
            delta_to_optimal_seconds: 0.0,
            monte_carlo_metrics: None,
        };

        let metrics = run_monte_carlo_simulation(&state, &dummy_strat, 100);
        assert!(metrics.expected_time_seconds > 0.0);
        assert!(metrics.p10_time_seconds <= metrics.expected_time_seconds);
        assert!(metrics.p90_time_seconds >= metrics.expected_time_seconds);
    }
}
