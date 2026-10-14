//! Multi-driver race simulation engine with reactive competitor strategies and on-track battles.
//!
//! Simulates multi-car interactions, undercut/overcut battles, dirty air hold-ups,
//! and strategic competitor counter-moves across the remaining race distance.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::TireCompound;
use crate::engine::odi::{calculate_dynamic_odi, calculate_overtake_probability, get_base_track_odi};
use crate::engine::physics::get_degradation_factor;

/// Single driver's strategy specification for multi-driver race simulation.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct DriverStrategyInput {
    /// Driver identifier or acronym (e.g. "NOR", "VER", "LEC", "HAM", "1", "44").
    pub driver_id: String,
    /// Constructor team name.
    #[serde(default)]
    pub team: Option<String>,
    /// Driver's current track position (1 = leader).
    #[serde(default)]
    pub current_position: Option<u32>,
    /// Current gap to leader in seconds (0.0 for leader).
    #[serde(default)]
    pub gap_to_leader_seconds: Option<f64>,
    /// Currently fitted tyre compound (e.g. "MEDIUM", "HARD", "SOFT").
    #[serde(default)]
    pub current_compound: Option<String>,
    /// Age of current tyre set in laps.
    #[serde(default)]
    pub tyre_age: Option<u32>,
    /// Planned pit lap(s) chosen by user or team engineer.
    #[serde(default)]
    pub planned_pit_laps: Vec<u32>,
    /// Target tyre compounds to fit on subsequent stints.
    #[serde(default)]
    pub target_compounds: Vec<String>,
    /// Strategic reaction mode: "Standard", "CoverUndercut", "ExtendStint", "AggressiveAttack".
    #[serde(default)]
    pub rival_reaction_mode: Option<String>,
}

/// Request payload for simulating multiple drivers simultaneously.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct MultiDriverSimRequest {
    /// Circuit identifier (e.g. "monza", "monaco", "silverstone", "spa").
    pub track_id: String,
    /// Total race laps (defaults to track standard if omitted).
    #[serde(default)]
    pub total_laps: Option<u32>,
    /// Current race lap when simulation begins.
    #[serde(default)]
    pub current_lap: Option<u32>,
    /// Base pit lane time loss in seconds (defaults to 22.5s).
    #[serde(default)]
    pub pit_loss_seconds: Option<f64>,
    /// List of driver strategy inputs.
    pub drivers: Vec<DriverStrategyInput>,
}

/// Simulated outcome for an individual driver in multi-driver race simulation.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct SimulatedDriverOutcome {
    pub driver_id: String,
    pub team: String,
    pub starting_position: u32,
    pub projected_finish_position: u32,
    pub position_delta: i32,
    pub projected_total_race_time_seconds: f64,
    pub delta_to_winner_seconds: f64,
    pub pit_stops_completed: u32,
    pub actual_pit_laps: Vec<u32>,
    pub tyre_stints: Vec<String>,
    pub overtakes_made: u32,
    pub reaction_notes: Vec<String>,
}

/// Overall response from multi-driver simulation.
#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
pub struct MultiDriverSimResponse {
    pub track_id: String,
    pub total_laps: u32,
    pub current_lap: u32,
    pub winner_driver_id: String,
    pub projected_podium: Vec<String>,
    pub results: Vec<SimulatedDriverOutcome>,
    pub race_narrative: String,
}

fn parse_compound(s: &str) -> TireCompound {
    match s.trim().to_uppercase().as_str() {
        "SOFT" => TireCompound::Soft,
        "HARD" => TireCompound::Hard,
        "INTERMEDIATE" => TireCompound::Intermediate,
        "WET" => TireCompound::Wet,
        _ => TireCompound::Medium,
    }
}

/// Simulates multi-driver race projection.
pub fn simulate_multi_driver_race(req: &MultiDriverSimRequest) -> MultiDriverSimResponse {
    let total_laps = req.total_laps.unwrap_or(53).max(1);
    let current_lap = req.current_lap.unwrap_or(20).min(total_laps);
    let base_pit_loss = req.pit_loss_seconds.unwrap_or(22.5);
    let base_track_odi = get_base_track_odi(&req.track_id);

    // If no drivers provided, generate a default competitive 5-driver scenario
    let mut driver_inputs = req.drivers.clone();
    if driver_inputs.is_empty() {
        driver_inputs = vec![
            DriverStrategyInput {
                driver_id: "NOR".to_string(),
                team: Some("McLaren".to_string()),
                current_position: Some(1),
                gap_to_leader_seconds: Some(0.0),
                current_compound: Some("MEDIUM".to_string()),
                tyre_age: Some(20),
                planned_pit_laps: vec![22],
                target_compounds: vec!["HARD".to_string()],
                rival_reaction_mode: Some("Standard".to_string()),
            },
            DriverStrategyInput {
                driver_id: "VER".to_string(),
                team: Some("Red Bull Racing".to_string()),
                current_position: Some(2),
                gap_to_leader_seconds: Some(2.1),
                current_compound: Some("MEDIUM".to_string()),
                tyre_age: Some(20),
                planned_pit_laps: vec![24],
                target_compounds: vec!["HARD".to_string()],
                rival_reaction_mode: Some("CoverUndercut".to_string()),
            },
            DriverStrategyInput {
                driver_id: "LEC".to_string(),
                team: Some("Ferrari".to_string()),
                current_position: Some(3),
                gap_to_leader_seconds: Some(5.4),
                current_compound: Some("HARD".to_string()),
                tyre_age: Some(20),
                planned_pit_laps: vec![35],
                target_compounds: vec!["MEDIUM".to_string()],
                rival_reaction_mode: Some("ExtendStint".to_string()),
            },
            DriverStrategyInput {
                driver_id: "HAM".to_string(),
                team: Some("Ferrari".to_string()),
                current_position: Some(4),
                gap_to_leader_seconds: Some(8.9),
                current_compound: Some("MEDIUM".to_string()),
                tyre_age: Some(20),
                planned_pit_laps: vec![25],
                target_compounds: vec!["HARD".to_string()],
                rival_reaction_mode: Some("Standard".to_string()),
            },
            DriverStrategyInput {
                driver_id: "PIA".to_string(),
                team: Some("McLaren".to_string()),
                current_position: Some(5),
                gap_to_leader_seconds: Some(12.3),
                current_compound: Some("MEDIUM".to_string()),
                tyre_age: Some(20),
                planned_pit_laps: vec![21, 38],
                target_compounds: vec!["HARD".to_string(), "SOFT".to_string()],
                rival_reaction_mode: Some("AggressiveAttack".to_string()),
            },
        ];
    }

    // Working state for each driver
    struct DriverSimState {
        driver_id: String,
        team: String,
        start_pos: u32,
        current_time: f64,
        compound: TireCompound,
        tyre_age: u32,
        planned_stops: Vec<u32>,
        target_compounds: Vec<TireCompound>,
        actual_pit_laps: Vec<u32>,
        reaction_mode: String,
        reaction_notes: Vec<String>,
        overtakes: u32,
        following_count: u32,
    }

    let mut states: Vec<DriverSimState> = driver_inputs
        .into_iter()
        .enumerate()
        .map(|(idx, d)| {
            let start_pos = d.current_position.unwrap_or((idx + 1) as u32);
            let gap = d.gap_to_leader_seconds.unwrap_or((start_pos as f64 - 1.0) * 2.0);
            let initial_compound = parse_compound(d.current_compound.as_deref().unwrap_or("MEDIUM"));
            let target_comps: Vec<TireCompound> = d
                .target_compounds
                .iter()
                .map(|c| parse_compound(c))
                .collect();

            DriverSimState {
                driver_id: d.driver_id,
                team: d.team.unwrap_or_else(|| "Formula 1".to_string()),
                start_pos,
                current_time: gap,
                compound: initial_compound,
                tyre_age: d.tyre_age.unwrap_or(0),
                planned_stops: d.planned_pit_laps,
                target_compounds: target_comps,
                actual_pit_laps: Vec::new(),
                reaction_mode: d.rival_reaction_mode.unwrap_or_else(|| "Standard".to_string()),
                reaction_notes: Vec::new(),
                overtakes: 0,
                following_count: 0,
            }
        })
        .collect();

    // Lap-by-lap simulation loop
    for lap in current_lap..=total_laps {
        // Track whether any driver pitted on this lap (for CoverUndercut triggers)
        let mut someone_pitted = false;

        // 1. Process pit stops
        for s in states.iter_mut() {
            let should_pit = s.planned_stops.contains(&lap);
            if should_pit {
                s.current_time += base_pit_loss;
                s.actual_pit_laps.push(lap);
                s.tyre_age = 0;
                if !s.target_compounds.is_empty() {
                    s.compound = s.target_compounds.remove(0);
                } else {
                    s.compound = TireCompound::Hard;
                }
                s.reaction_notes.push(format!(
                    "L{}: Pit stop executed -> fitted {:?}",
                    lap, s.compound
                ));
                someone_pitted = true;
            }
        }

        // 2. Process reactive strategy triggers for the next lap
        if someone_pitted {
            for s in states.iter_mut() {
                if s.reaction_mode == "CoverUndercut" && !s.actual_pit_laps.contains(&lap) {
                    // Check if they have an upcoming stop within 3 laps; if so, trigger immediately next lap
                    if let Some(pos) = s.planned_stops.iter().position(|&p| p > lap && p <= lap + 3) {
                        s.planned_stops[pos] = lap + 1;
                        s.reaction_notes.push(format!(
                            "L{}: Reactive trigger - covering rival undercut by moving pit stop to L{}",
                            lap, lap + 1
                        ));
                    }
                }
            }
        }

        // 3. Compute lap pace for each driver
        for s in states.iter_mut() {
            s.tyre_age += 1;
            let deg = get_degradation_factor(&s.compound, s.tyre_age);
            let fuel_burn = (lap as f64) * -0.03;
            let base_pace = 89.0 + deg + fuel_burn;
            s.current_time += base_pace;
        }

        // 4. On-track battles and overtaking evaluation
        // Sort cars by current cumulative race time to determine on-track proximity
        states.sort_by(|a, b| a.current_time.partial_cmp(&b.current_time).unwrap_or(std::cmp::Ordering::Equal));

        for i in 1..states.len() {
            let gap_to_ahead = states[i].current_time - states[i - 1].current_time;
            if gap_to_ahead > 0.0 && gap_to_ahead <= 1.0 {
                // Car i is inside DRS range of car i-1
                states[i].following_count += 1;
                let odi_res = calculate_dynamic_odi(
                    base_track_odi,
                    lap,
                    total_laps,
                    states[i].following_count,
                    0.0,
                );

                let pace_delta = 0.6; // benchmark pace advantage needed
                let overtake_prob = calculate_overtake_probability(odi_res.dynamic_odi, pace_delta);

                let rival_id = states[i - 1].driver_id.clone();
                if overtake_prob > 0.55 {
                    // Overtake executed! Swap positions slightly
                    states[i].overtakes += 1;
                    states[i].reaction_notes.push(format!(
                        "L{}: Overtook {} (Dynamic ODI: {:.2}, Prob: {:.0}%)",
                        lap,
                        rival_id,
                        odi_res.dynamic_odi,
                        overtake_prob * 100.0
                    ));
                    // Small delta advantage for clean pass
                    states[i].current_time -= 0.4;
                    states[i].following_count = 0;
                } else {
                    // Held up in dirty air
                    states[i].current_time += 0.5 * (1.0 + odi_res.dynamic_odi);
                }
            } else {
                states[i].following_count = 0;
            }
        }
    }

    // Final sorting by total race time
    states.sort_by(|a, b| a.current_time.partial_cmp(&b.current_time).unwrap_or(std::cmp::Ordering::Equal));

    let winner_time = states.first().map(|s| s.current_time).unwrap_or(0.0);
    let winner_id = states.first().map(|s| s.driver_id.clone()).unwrap_or_default();
    let podium: Vec<String> = states.iter().take(3).map(|s| s.driver_id.clone()).collect();

    let results: Vec<SimulatedDriverOutcome> = states
        .into_iter()
        .enumerate()
        .map(|(idx, s)| {
            let finish_pos = (idx + 1) as u32;
            let delta_winner = s.current_time - winner_time;
            let pos_delta = s.start_pos as i32 - finish_pos as i32; // positive = gained positions

            SimulatedDriverOutcome {
                driver_id: s.driver_id,
                team: s.team,
                starting_position: s.start_pos,
                projected_finish_position: finish_pos,
                position_delta: pos_delta,
                projected_total_race_time_seconds: (s.current_time * 100.0).round() / 100.0,
                delta_to_winner_seconds: (delta_winner * 100.0).round() / 100.0,
                pit_stops_completed: s.actual_pit_laps.len() as u32,
                actual_pit_laps: s.actual_pit_laps,
                tyre_stints: vec![format!("{:?}", s.compound)],
                overtakes_made: s.overtakes,
                reaction_notes: s.reaction_notes,
            }
        })
        .collect();

    let narrative = format!(
        "Race simulation completed for {} laps. {} wins the race. Projected podium: {}. Track dynamic ODI baseline was {:.2}.",
        total_laps,
        winner_id,
        podium.join(", "),
        base_track_odi
    );

    MultiDriverSimResponse {
        track_id: req.track_id.clone(),
        total_laps,
        current_lap,
        winner_driver_id: winner_id,
        projected_podium: podium,
        results,
        race_narrative: narrative,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_driver_simulation_basic() {
        let req = MultiDriverSimRequest {
            track_id: "monza".to_string(),
            total_laps: Some(53),
            current_lap: Some(20),
            pit_loss_seconds: Some(22.5),
            drivers: vec![],
        };

        let resp = simulate_multi_driver_race(&req);
        assert_eq!(resp.results.len(), 5);
        assert_eq!(resp.results[0].projected_finish_position, 1);
        assert_eq!(resp.results[0].delta_to_winner_seconds, 0.0);
        assert_eq!(resp.projected_podium.len(), 3);
    }
}
