//! Lap-by-lap vehicle physics, tire wear curves, dirty air, fuel burn, and pit loss simulation engine.

use crate::domain::{CompetitorCar, TireCompound, TrackStatus, WeatherForecast, WeatherState};

/// Calculates per-lap tire degradation pace loss in seconds.
///
/// Models linear degradation up to 80% of tire compound lifecycle, after which
/// an exponential penalty curve ("tire cliff") is applied.
///
/// # Arguments
/// * `compound` - Currently fitted tire compound.
/// * `age` - Cumulative tire age in laps.
pub fn get_degradation_factor(compound: &TireCompound, age: u32) -> f64 {
    let base_rate = match compound {
        TireCompound::Soft => 0.15,
        TireCompound::Medium => 0.08,
        TireCompound::Hard => 0.04,
        TireCompound::Intermediate => 0.25,
        TireCompound::Wet => 0.30,
    };

    let lifecycle = compound.lifecycle_laps() as f64;
    let cliff_lap = lifecycle * 0.8;

    let mut deg = base_rate * (age as f64);

    if (age as f64) > cliff_lap {
        let over = (age as f64) - cliff_lap;
        deg += (over * 0.2).exp() - 1.0;
    }

    deg
}

/// Evaluates total pit stop time loss taking into account Safety Car / VSC discounts, front wing replacement, and penalties.
///
/// # Arguments
/// * `base_loss` - Standard stationary & pit lane transit loss under Green flag (seconds).
/// * `environment` - Current track neutralization flag status.
/// * `front_wing_damage` - True if nose cone / front wing replacement is required (+10.0s).
/// * `time_penalty_seconds` - Accrued steward time penalty to be served in pit box (seconds).
/// * `is_pit_on_current_lap` - True if pit stop occurs during current lap (applies active SC/VSC discount).
///
/// # Returns
/// A tuple containing `(total_pit_loss_seconds, explanation_reasons)`.
pub fn calculate_pit_stop_loss(
    base_loss: f64,
    environment: &TrackStatus,
    front_wing_damage: bool,
    time_penalty_seconds: f64,
    is_pit_on_current_lap: bool,
) -> (f64, Vec<String>) {
    let mut reasons = Vec::new();
    let mut pit_loss = base_loss;

    if is_pit_on_current_lap {
        match environment {
            TrackStatus::SafetyCar => {
                pit_loss *= 0.60;
                reasons.push(format!(
                    "Safety Car cheap pit stop discount (-{:.1}s)",
                    base_loss * 0.40
                ));
            }
            TrackStatus::VirtualSafetyCar => {
                pit_loss *= 0.70;
                reasons.push(format!(
                    "VSC cheap pit stop discount (-{:.1}s)",
                    base_loss * 0.30
                ));
            }
            _ => {
                reasons.push("Standard pit stop loss".to_string());
            }
        }
    } else {
        reasons.push("Standard pit stop loss".to_string());
    }

    if front_wing_damage {
        pit_loss += 10.0;
        reasons.push("Front wing replacement (+10.0s)".to_string());
    }

    if time_penalty_seconds > 0.0 {
        pit_loss += time_penalty_seconds;
        reasons.push(format!(
            "Served time penalty (+{:.1}s)",
            time_penalty_seconds
        ));
    }

    (pit_loss, reasons)
}

/// Evaluates active weather condition for a specific lap based on forecast transition lap.
pub fn get_weather_for_lap(lap: u32, forecast: &Option<WeatherForecast>) -> WeatherState {
    if let Some(f) = forecast {
        if lap >= f.transition_lap {
            f.predicted_weather.clone()
        } else {
            f.current_weather.clone()
        }
    } else {
        WeatherState::Dry
    }
}

/// Simulates stint duration and cumulative tire degradation across a specified lap range.
///
/// Computes lap time additions for base pace (90.0s), tire wear, fuel load decay (-0.03s/lap burned),
/// track rubbering (-0.015s/lap), dirty air penalty (+1.0s within 1.5s gap), weather mismatch penalties,
/// and Safety Car pace deltas (+30.0s).
///
/// # Returns
/// A tuple `(total_stint_time_seconds, cumulative_degradation_loss_seconds)`.
pub fn simulate_stint(
    start_lap: u32,
    end_lap: u32,
    compound: &TireCompound,
    initial_tire_age: u32,
    environment: &TrackStatus,
    competitors: &[CompetitorCar],
    weather_forecast: &Option<WeatherForecast>,
) -> (f64, f64) {
    if start_lap >= end_lap {
        return (0.0, 0.0);
    }

    let mut total_stint_time = 0.0;
    let mut total_deg_loss = 0.0;
    for (stint_lap_idx, lap) in (start_lap..end_lap).enumerate() {
        let age = initial_tire_age + stint_lap_idx as u32;
        let deg = get_degradation_factor(compound, age);
        let fuel_effect = (lap as f64) * -0.03;
        let track_evolution = (lap as f64) * -0.015;

        let mut dirty_air_penalty = 0.0;
        for comp in competitors {
            if comp.gap_to_ego_seconds > 0.0
                && comp.gap_to_ego_seconds <= 1.5
                && lap < comp.projected_pit_lap
            {
                dirty_air_penalty = 1.0;
                break;
            }
        }

        let weather = get_weather_for_lap(lap, weather_forecast);
        let mut weather_penalty = 0.0;

        match weather {
            WeatherState::Dry => match compound {
                TireCompound::Intermediate => weather_penalty += 10.0,
                TireCompound::Wet => weather_penalty += 20.0,
                _ => {}
            },
            WeatherState::Damp => match compound {
                TireCompound::Soft | TireCompound::Medium | TireCompound::Hard => {
                    weather_penalty += 15.0
                }
                TireCompound::Intermediate => weather_penalty += 0.0,
                TireCompound::Wet => weather_penalty += 5.0,
            },
            WeatherState::Wet => match compound {
                TireCompound::Soft | TireCompound::Medium | TireCompound::Hard => {
                    weather_penalty += 30.0
                }
                TireCompound::Intermediate => weather_penalty += 5.0,
                TireCompound::Wet => weather_penalty += 0.0,
            },
        }

        let lap_time = match environment {
            TrackStatus::RedFlag => 0.0,
            TrackStatus::SafetyCar | TrackStatus::VirtualSafetyCar => {
                90.0 + deg
                    + 30.0
                    + fuel_effect
                    + track_evolution
                    + dirty_air_penalty
                    + weather_penalty
            }
            TrackStatus::Green | TrackStatus::Yellow => {
                90.0 + deg + fuel_effect + track_evolution + dirty_air_penalty + weather_penalty
            }
        };

        if *environment != TrackStatus::RedFlag {
            total_deg_loss += deg;
        }

        total_stint_time += lap_time;
    }

    (total_stint_time, total_deg_loss)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_stint() {
        let (time, deg) = simulate_stint(
            1,
            10,
            &TireCompound::Soft,
            0,
            &TrackStatus::Green,
            &[],
            &None,
        );
        assert!(time > 0.0);
        assert!(deg > 0.0);
    }

    #[test]
    fn test_pit_stop_loss_penalties_damage() {
        let (loss, reasons) = calculate_pit_stop_loss(20.0, &TrackStatus::Green, true, 5.0, false);
        assert_eq!(loss, 20.0 + 10.0 + 5.0);
        assert_eq!(reasons.len(), 3);
    }

    #[test]
    fn test_safety_car_pit_loss() {
        let (loss, _) = calculate_pit_stop_loss(20.0, &TrackStatus::SafetyCar, false, 0.0, true);
        assert_eq!(loss, 12.0); // 20.0 * 0.6
    }

    #[test]
    fn test_vsc_pit_loss() {
        let (loss, _) =
            calculate_pit_stop_loss(20.0, &TrackStatus::VirtualSafetyCar, false, 0.0, true);
        assert_eq!(loss, 14.0); // 20.0 * 0.7
    }
}
