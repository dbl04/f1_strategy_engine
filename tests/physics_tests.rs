use f1_strategy_engine::domain::{EgoCar, TireCompound, TrackStatus};
use f1_strategy_engine::engine::physics::{calculate_pit_stop_loss, get_degradation_factor};
use f1_strategy_engine::engine::regulations::is_valid_f1_strategy;

#[test]
fn test_tire_cliff_degradation() {
    let soft_early = get_degradation_factor(&TireCompound::Soft, 5);
    let soft_late = get_degradation_factor(&TireCompound::Soft, 25);
    assert!(soft_late > soft_early * 4.0);
}

#[test]
fn test_fia_compound_diversity_regulation() {
    let ego = EgoCar {
        current_lap: 10,
        current_tire: TireCompound::Soft,
        tire_age_laps: 10,
        mandatory_pit_completed: false,
        front_wing_damage: false,
        time_penalty_seconds: 0.0,
    };

    assert!(!is_valid_f1_strategy(&ego, &[TireCompound::Soft]));
    assert!(is_valid_f1_strategy(&ego, &[TireCompound::Medium]));
}

#[test]
fn test_safety_car_pit_discount() {
    let (loss_sc, _) = calculate_pit_stop_loss(20.0, &TrackStatus::SafetyCar, false, 0.0, true);
    let (loss_green, _) = calculate_pit_stop_loss(20.0, &TrackStatus::Green, false, 0.0, true);
    assert!(loss_sc < loss_green);
}

#[test]
fn test_monte_carlo_10k_benchmark() {
    use f1_strategy_engine::domain::{RaceState, StrategyOption, TrackParameters};
    use f1_strategy_engine::engine::monte_carlo::run_monte_carlo_simulation;

    let state = RaceState {
        track: TrackParameters {
            total_laps: 53,
            base_pit_stop_loss_seconds: 22.0,
            historical_sc_probability: 0.35,
        },
        environment: TrackStatus::Green,
        ego_car: EgoCar {
            current_lap: 12,
            current_tire: TireCompound::Medium,
            tire_age_laps: 12,
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
        projected_total_time_seconds: 4800.0,
        projected_total_time_formatted: "1h 20m 0.0s".to_string(),
        average_lap_time_seconds: 90.5,
        total_degradation_loss_seconds: 25.0,
        total_pit_stop_time_loss_seconds: 22.0,
        delta_to_optimal_seconds: 0.0,
        monte_carlo_metrics: None,
    };

    let start = std::time::Instant::now();
    let metrics = run_monte_carlo_simulation(&state, &dummy_strat, 10_000);
    let duration = start.elapsed();

    println!("\n⏱️ [BENCHMARK] 10,000 Monte Carlo simulations completed in: {:?}", duration);
    println!("📊 Expected: {}s, P10: {}s, P90: {}s, Win%: {}%\n",
        metrics.expected_time_seconds, metrics.p10_time_seconds, metrics.p90_time_seconds, metrics.win_probability_pct);

    assert!(duration.as_millis() < 100, "Should take under 100ms! Took {:?}", duration);
}

