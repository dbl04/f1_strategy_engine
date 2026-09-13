//! Multi-stint strategy permutation generator and race optimization engine.

use crate::domain::{
    MultiStrategyResponse, PitStopPlan, RaceState, StrategyOption, TireCompound, TrackStatus,
    TrafficWindowResponse, UndercutOvercutOption,
};
use crate::engine::physics::{calculate_pit_stop_loss, simulate_stint};
use crate::engine::regulations::is_valid_f1_strategy;

/// Formats a time duration in seconds into a human-readable race telemetry string (e.g. `"1h 24m 12.5s"`).
pub fn format_duration(seconds: f64) -> String {
    let total_secs = seconds.round() as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = seconds % 60.0;

    if hours > 0 {
        format!("{}h {}m {:.1}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {:.1}s", minutes, secs)
    } else {
        format!("{:.1}s", secs)
    }
}

/// Evaluates all candidate strategy permutations (0-stop, 1-stop, 2-stop) for a given race state,
/// ranks them by overall projected race completion time while enforcing FIA sporting regulations,
/// and computes undercut/overcut matrices and pit exit traffic windows.
///
/// # Arguments
/// * `state` - Current race snapshot state containing track config, flag conditions, ego car telemetry, and rival competitors.
///
/// # Returns
/// A [`MultiStrategyResponse`] containing the optimal strategy, top alternative strategies,
/// undercut/overcut options, traffic window release warnings, and safety car probability metrics.
pub fn optimize_race_strategies(state: &RaceState) -> MultiStrategyResponse {
    let current_lap = state.ego_car.current_lap;
    let total_laps = state.track.total_laps;
    let laps_remaining = total_laps.saturating_sub(current_lap);

    let sc_active = matches!(
        state.environment,
        TrackStatus::SafetyCar | TrackStatus::VirtualSafetyCar
    );

    if laps_remaining == 0 {
        let dummy = StrategyOption {
            name: "Race Finished".to_string(),
            is_valid_f1_rules: true,
            total_pit_stops: 0,
            pit_stops: vec![],
            projected_total_time_seconds: 0.0,
            projected_total_time_formatted: "0s".to_string(),
            average_lap_time_seconds: 0.0,
            total_degradation_loss_seconds: 0.0,
            total_pit_stop_time_loss_seconds: 0.0,
            delta_to_optimal_seconds: 0.0,
            monte_carlo_metrics: None,
        };

        return MultiStrategyResponse {
            optimal_strategy: dummy,
            alternative_strategies: vec![],
            safety_car_advantage_active: sc_active,
            summary_message: "Race completed. No laps remaining to simulate.".to_string(),
            undercut_overcut_options: vec![],
            traffic_windows: vec![],
            safety_car_probability_per_lap: 0.0,
        };
    }

    let candidate_compounds = match state.ego_car.current_tire {
        TireCompound::Intermediate | TireCompound::Wet => vec![
            TireCompound::Intermediate,
            TireCompound::Wet,
            TireCompound::Medium,
        ],
        _ => vec![
            TireCompound::Soft,
            TireCompound::Medium,
            TireCompound::Hard,
        ],
    };

    let mut evaluated_strategies: Vec<StrategyOption> = Vec::new();

    // 1. 0-Stop
    let (time_0stop, deg_0stop) = simulate_stint(
        current_lap,
        total_laps,
        &state.ego_car.current_tire,
        state.ego_car.tire_age_laps,
        &state.environment,
        &state.competitors,
        &state.weather_forecast,
    );

    let is_valid_0stop = is_valid_f1_strategy(&state.ego_car, &[]);

    evaluated_strategies.push(StrategyOption {
        name: format!("0-Stop: {:?} to end", state.ego_car.current_tire),
        is_valid_f1_rules: is_valid_0stop,
        total_pit_stops: 0,
        pit_stops: vec![],
        projected_total_time_seconds: time_0stop,
        projected_total_time_formatted: format_duration(time_0stop),
        average_lap_time_seconds: (time_0stop / laps_remaining as f64 * 100.0).round() / 100.0,
        total_degradation_loss_seconds: (deg_0stop * 100.0).round() / 100.0,
        total_pit_stop_time_loss_seconds: 0.0,
        delta_to_optimal_seconds: 0.0,
        monte_carlo_metrics: None,
    });

    // 2. 1-Stop
    let pit_step = if laps_remaining > 30 { 2 } else { 1 };

    for pit_lap in (current_lap + 1..total_laps).step_by(pit_step) {
        let is_pit_current_lap = pit_lap == current_lap + 1;

        let (pit_loss, pit_reasons) = calculate_pit_stop_loss(
            state.track.base_pit_stop_loss_seconds,
            &state.environment,
            state.ego_car.front_wing_damage,
            state.ego_car.time_penalty_seconds,
            is_pit_current_lap,
        );

        for new_compound in &candidate_compounds {
            let (stint1_time, stint1_deg) = simulate_stint(
                current_lap,
                pit_lap,
                &state.ego_car.current_tire,
                state.ego_car.tire_age_laps,
                &state.environment,
                &state.competitors,
                &state.weather_forecast,
            );

            let (stint2_time, stint2_deg) = simulate_stint(
                pit_lap,
                total_laps,
                new_compound,
                0,
                &state.environment,
                &state.competitors,
                &state.weather_forecast,
            );

            let total_time = stint1_time + pit_loss + stint2_time;
            let total_deg = stint1_deg + stint2_deg;
            let is_valid = is_valid_f1_strategy(&state.ego_car, std::slice::from_ref(new_compound));

            let pit_plan = PitStopPlan {
                pit_lap,
                new_compound: new_compound.clone(),
                pit_loss_seconds: (pit_loss * 100.0).round() / 100.0,
                reasons: pit_reasons.clone(),
            };

            let name_extra = if is_pit_current_lap && sc_active {
                " (SC Box Now!)"
            } else {
                ""
            };

            evaluated_strategies.push(StrategyOption {
                name: format!(
                    "1-Stop: {:?} (L{}) ➔ {:?}{}",
                    state.ego_car.current_tire, pit_lap, new_compound, name_extra
                ),
                is_valid_f1_rules: is_valid,
                total_pit_stops: 1,
                pit_stops: vec![pit_plan],
                projected_total_time_seconds: total_time,
                projected_total_time_formatted: format_duration(total_time),
                average_lap_time_seconds: (total_time / laps_remaining as f64 * 100.0).round()
                    / 100.0,
                total_degradation_loss_seconds: (total_deg * 100.0).round() / 100.0,
                total_pit_stop_time_loss_seconds: (pit_loss * 100.0).round() / 100.0,
                delta_to_optimal_seconds: 0.0,
                monte_carlo_metrics: None,
            });
        }
    }

    // 3. 2-Stop
    if laps_remaining >= 15 {
        let p1_lap = current_lap + (laps_remaining / 3);
        let p2_lap = current_lap + (2 * laps_remaining / 3);

        if p1_lap > current_lap && p2_lap > p1_lap && p2_lap < total_laps {
            let (pit1_loss, pit1_reasons) = calculate_pit_stop_loss(
                state.track.base_pit_stop_loss_seconds,
                &state.environment,
                state.ego_car.front_wing_damage,
                state.ego_car.time_penalty_seconds,
                p1_lap == current_lap + 1,
            );

            let (pit2_loss, pit2_reasons) = calculate_pit_stop_loss(
                state.track.base_pit_stop_loss_seconds,
                &TrackStatus::Green,
                false,
                0.0,
                false,
            );

            for c1 in &candidate_compounds {
                for c2 in &candidate_compounds {
                    let (s1_t, s1_d) = simulate_stint(
                        current_lap,
                        p1_lap,
                        &state.ego_car.current_tire,
                        state.ego_car.tire_age_laps,
                        &state.environment,
                        &state.competitors,
                        &state.weather_forecast,
                    );
                    let (s2_t, s2_d) = simulate_stint(
                        p1_lap,
                        p2_lap,
                        c1,
                        0,
                        &state.environment,
                        &state.competitors,
                        &state.weather_forecast,
                    );
                    let (s3_t, s3_d) = simulate_stint(
                        p2_lap,
                        total_laps,
                        c2,
                        0,
                        &state.environment,
                        &state.competitors,
                        &state.weather_forecast,
                    );

                    let total_time = s1_t + pit1_loss + s2_t + pit2_loss + s3_t;
                    let total_deg = s1_d + s2_d + s3_d;
                    let total_pit_loss = pit1_loss + pit2_loss;

                    let pit1 = PitStopPlan {
                        pit_lap: p1_lap,
                        new_compound: c1.clone(),
                        pit_loss_seconds: (pit1_loss * 100.0).round() / 100.0,
                        reasons: pit1_reasons.clone(),
                    };

                    let pit2 = PitStopPlan {
                        pit_lap: p2_lap,
                        new_compound: c2.clone(),
                        pit_loss_seconds: (pit2_loss * 100.0).round() / 100.0,
                        reasons: pit2_reasons.clone(),
                    };

                    let is_valid = is_valid_f1_strategy(&state.ego_car, &[c1.clone(), c2.clone()]);

                    evaluated_strategies.push(StrategyOption {
                        name: format!(
                            "2-Stop: {:?} (L{}) ➔ {:?} (L{}) ➔ {:?}",
                            state.ego_car.current_tire, p1_lap, c1, p2_lap, c2
                        ),
                        is_valid_f1_rules: is_valid,
                        total_pit_stops: 2,
                        pit_stops: vec![pit1, pit2],
                        projected_total_time_seconds: total_time,
                        projected_total_time_formatted: format_duration(total_time),
                        average_lap_time_seconds: (total_time / laps_remaining as f64 * 100.0)
                            .round()
                            / 100.0,
                        total_degradation_loss_seconds: (total_deg * 100.0).round() / 100.0,
                        total_pit_stop_time_loss_seconds: (total_pit_loss * 100.0).round() / 100.0,
                        delta_to_optimal_seconds: 0.0,
                        monte_carlo_metrics: None,
                    });
                }
            }
        }
    }

    evaluated_strategies.sort_by(|a, b| {
        match (a.is_valid_f1_rules, b.is_valid_f1_rules) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a
                .projected_total_time_seconds
                .partial_cmp(&b.projected_total_time_seconds)
                .unwrap_or(std::cmp::Ordering::Equal),
        }
    });

    let mut optimal = evaluated_strategies[0].clone();
    optimal.monte_carlo_metrics = Some(crate::engine::monte_carlo::run_monte_carlo_simulation(state, &optimal, 5000));
    let min_time = optimal.projected_total_time_seconds;

    let mut alternatives: Vec<StrategyOption> = Vec::new();
    for strat in evaluated_strategies.into_iter().skip(1).take(5) {
        let mut alt = strat;
        alt.delta_to_optimal_seconds =
            ((alt.projected_total_time_seconds - min_time) * 100.0).round() / 100.0;
        alt.monte_carlo_metrics = Some(crate::engine::monte_carlo::run_monte_carlo_simulation(state, &alt, 1000));
        alternatives.push(alt);
    }


    let summary = format!(
        "Optimal Strategy: '{}' with projected time of {}. (Simulated {} laps remaining).",
        optimal.name, optimal.projected_total_time_formatted, laps_remaining
    );

    let mut undercut_overcut_options = Vec::new();
    let candidate_new_compound = match state.ego_car.current_tire {
        TireCompound::Soft => TireCompound::Medium,
        TireCompound::Medium => TireCompound::Hard,
        TireCompound::Hard => TireCompound::Medium,
        TireCompound::Intermediate => TireCompound::Wet,
        TireCompound::Wet => TireCompound::Intermediate,
    };

    for comp in &state.competitors {
        let target_lap = comp.projected_pit_lap;
        if target_lap <= current_lap || target_lap >= total_laps {
            continue;
        }

        let baseline_s1 = simulate_stint(
            current_lap,
            target_lap,
            &state.ego_car.current_tire,
            state.ego_car.tire_age_laps,
            &state.environment,
            &state.competitors,
            &state.weather_forecast,
        )
        .0;
        let baseline_s2 = simulate_stint(
            target_lap,
            total_laps,
            &candidate_new_compound,
            0,
            &state.environment,
            &state.competitors,
            &state.weather_forecast,
        )
        .0;
        let baseline_total = baseline_s1 + baseline_s2;

        for delta in [-3, -2, -1, 1, 2, 3] {
            let ego_pit_lap = (target_lap as i32 + delta) as u32;
            if ego_pit_lap > current_lap && ego_pit_lap < total_laps {
                let s1 = simulate_stint(
                    current_lap,
                    ego_pit_lap,
                    &state.ego_car.current_tire,
                    state.ego_car.tire_age_laps,
                    &state.environment,
                    &state.competitors,
                    &state.weather_forecast,
                )
                .0;
                let s2 = simulate_stint(
                    ego_pit_lap,
                    total_laps,
                    &candidate_new_compound,
                    0,
                    &state.environment,
                    &state.competitors,
                    &state.weather_forecast,
                )
                .0;
                let total = s1 + s2;

                let advantage = baseline_total - total;

                undercut_overcut_options.push(UndercutOvercutOption {
                    competitor_name: comp.driver_name.clone(),
                    target_pit_lap: target_lap,
                    pit_lap_delta: delta,
                    ego_pit_lap,
                    advantage_seconds: (advantage * 100.0).round() / 100.0,
                });
            }
        }
    }

    let mut traffic_windows = Vec::new();
    for pit_lap in current_lap + 1..=(current_lap + 5).min(total_laps) {
        let (pit_loss, _) = calculate_pit_stop_loss(
            state.track.base_pit_stop_loss_seconds,
            &state.environment,
            state.ego_car.front_wing_damage,
            state.ego_car.time_penalty_seconds,
            pit_lap == current_lap + 1,
        );

        let mut clean_air = true;
        let mut warning = "Clean air pit exit".to_string();

        for comp in &state.competitors {
            let comp_loss = if comp.projected_pit_lap < pit_lap {
                pit_loss
            } else {
                0.0
            };
            let projected_gap = comp.gap_to_ego_seconds + pit_loss - comp_loss;

            if projected_gap.abs() < 2.5 {
                clean_air = false;
                warning = format!(
                    "Traffic warning: will emerge into DRS train with {}",
                    comp.driver_name
                );
                break;
            }
        }

        traffic_windows.push(TrafficWindowResponse {
            pit_lap,
            is_clean_air: clean_air,
            traffic_warning: warning,
        });
    }

    let sc_prob_per_lap = if total_laps > 0 {
        state.track.historical_sc_probability / (total_laps as f64)
    } else {
        0.0
    };

    MultiStrategyResponse {
        optimal_strategy: optimal,
        alternative_strategies: alternatives,
        safety_car_advantage_active: sc_active,
        summary_message: summary,
        undercut_overcut_options,
        traffic_windows,
        safety_car_probability_per_lap: sc_prob_per_lap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EgoCar, TrackParameters};

    #[test]
    fn test_optimize_race_strategies_valid_0_stop() {
        let state = RaceState {
            track: TrackParameters {
                total_laps: 50,
                base_pit_stop_loss_seconds: 20.0,
                historical_sc_probability: 0.0,
            },
            environment: TrackStatus::Green,
            ego_car: EgoCar {
                current_lap: 45,
                current_tire: TireCompound::Hard,
                tire_age_laps: 5,
                mandatory_pit_completed: true,
                front_wing_damage: false,
                time_penalty_seconds: 0.0,
            },
            competitors: vec![],
            weather_forecast: None,
        };
        let response = optimize_race_strategies(&state);
        assert!(response.optimal_strategy.is_valid_f1_rules);
    }

    #[test]
    fn test_optimize_race_strategies_invalid_compound() {
        let state = RaceState {
            track: TrackParameters {
                total_laps: 50,
                base_pit_stop_loss_seconds: 20.0,
                historical_sc_probability: 0.0,
            },
            environment: TrackStatus::Green,
            ego_car: EgoCar {
                current_lap: 45,
                current_tire: TireCompound::Hard,
                tire_age_laps: 5,
                mandatory_pit_completed: false,
                front_wing_damage: false,
                time_penalty_seconds: 0.0,
            },
            competitors: vec![],
            weather_forecast: None,
        };
        let response = optimize_race_strategies(&state);
        assert!(response.optimal_strategy.is_valid_f1_rules);
        assert!(response.optimal_strategy.total_pit_stops > 0);
    }
}

