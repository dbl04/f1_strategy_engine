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
