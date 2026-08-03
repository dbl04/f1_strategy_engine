use axum::{
    routing::{get, post},
    response::Html,
    Router,
    Json,
    extract::Query,
};
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Serve the interactive F1 Strategy Engine Dashboard UI at GET /
async fn index_handler() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

// Circuit Information metadata including OSM query name for Overpass API geometry fetching.
#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
pub struct CircuitInfo {
    pub id: String,
    pub name: String,
    pub country: String,
    pub flag_emoji: String,
    pub total_laps: u32,
    pub base_pit_stop_loss_seconds: f64,
    pub length_km: f64,
    pub osm_query_name: String,
}

/// Get list of available F1 circuits and telemetry data.
#[utoipa::path(
    get,
    path = "/api/tracks",
    responses(
        (status = 200, description = "List of official F1 circuits", body = Vec<CircuitInfo>)
    )
)]
async fn get_tracks() -> Json<Vec<CircuitInfo>> {
    let tracks = vec![
        CircuitInfo {
            id: "monza".to_string(),
            name: "Autodromo Nazionale Monza".to_string(),
            country: "Italy".to_string(),
            flag_emoji: "🇮🇹".to_string(),
            total_laps: 53,
            base_pit_stop_loss_seconds: 25.0,
            length_km: 5.793,
            osm_query_name: "Autodromo Nazionale Monza".to_string(),
        },
        CircuitInfo {
            id: "silverstone".to_string(),
            name: "Silverstone Circuit".to_string(),
            country: "UK".to_string(),
            flag_emoji: "🇬🇧".to_string(),
            total_laps: 52,
            base_pit_stop_loss_seconds: 20.5,
            length_km: 5.891,
            osm_query_name: "Silverstone Circuit".to_string(),
        },
        CircuitInfo {
            id: "spa".to_string(),
            name: "Circuit de Spa-Francorchamps".to_string(),
            country: "Belgium".to_string(),
            flag_emoji: "🇧🇪".to_string(),
            total_laps: 44,
            base_pit_stop_loss_seconds: 22.0,
            length_km: 7.004,
            osm_query_name: "Circuit de Spa-Francorchamps".to_string(),
        },
        CircuitInfo {
            id: "monaco".to_string(),
            name: "Circuit de Monaco".to_string(),
            country: "Monaco".to_string(),
            flag_emoji: "🇲🇨".to_string(),
            total_laps: 78,
            base_pit_stop_loss_seconds: 19.5,
            length_km: 3.337,
            osm_query_name: "Circuit de Monaco".to_string(),
        },
        CircuitInfo {
            id: "bahrain".to_string(),
            name: "Bahrain International Circuit".to_string(),
            country: "Bahrain".to_string(),
            flag_emoji: "🇧🇭".to_string(),
            total_laps: 57,
            base_pit_stop_loss_seconds: 22.5,
            length_km: 5.412,
            osm_query_name: "Bahrain International Circuit".to_string(),
        },
        CircuitInfo {
            id: "suzuka".to_string(),
            name: "Suzuka International Racing Course".to_string(),
            country: "Japan".to_string(),
            flag_emoji: "🇯🇵".to_string(),
            total_laps: 53,
            base_pit_stop_loss_seconds: 22.5,
            length_km: 5.807,
            osm_query_name: "Suzuka Circuit".to_string(),
        },
        CircuitInfo {
            id: "cota".to_string(),
            name: "Circuit of the Americas".to_string(),
            country: "USA".to_string(),
            flag_emoji: "🇺🇸".to_string(),
            total_laps: 56,
            base_pit_stop_loss_seconds: 20.0,
            length_km: 5.513,
            osm_query_name: "Circuit of the Americas".to_string(),
        },
        CircuitInfo {
            id: "singapore".to_string(),
            name: "Marina Bay Street Circuit".to_string(),
            country: "Singapore".to_string(),
            flag_emoji: "🇸🇬".to_string(),
            total_laps: 62,
            base_pit_stop_loss_seconds: 28.0,
            length_km: 4.940,
            osm_query_name: "Marina Bay Street Circuit".to_string(),
        },
    ];

    Json(tracks)
}

// Query parameters for the Overpass API proxy endpoint.
#[derive(Deserialize, utoipa::IntoParams)]
pub struct OverpassQuery {
    pub name: String,
}

/// Proxy endpoint to fetch real circuit geometry from OpenStreetMap Overpass API.
/// Avoids browser CORS restrictions by proxying the request server-side.
#[utoipa::path(
    get,
    path = "/api/circuit-geometry",
    params(OverpassQuery),
    responses(
        (status = 200, description = "Raw Overpass API JSON response with circuit geometry")
    )
)]
async fn get_circuit_geometry(Query(params): Query<OverpassQuery>) -> Json<serde_json::Value> {
    let overpass_query = format!(
        r#"[out:json][timeout:25];(relation["name"~"{}",i];way["highway"="raceway"]["name"~"{}",i];way["name"~"{}",i]["highway"];);(._;>;);out geom;"#,
        params.name, params.name, params.name
    );

    let url = format!(
        "https://overpass-api.de/api/interpreter?data={}",
        urlencoding(&overpass_query)
    );

    match reqwest::get(&url).await {
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Ok(json) => Json(json),
                Err(_) => Json(serde_json::json!({"error": "Failed to parse Overpass response", "elements": []})),
            }
        }
        Err(e) => {
            Json(serde_json::json!({"error": format!("Overpass API request failed: {}", e), "elements": []}))
        }
    }
}

fn urlencoding(s: &str) -> String {
    s.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
            format!("{}", b as char)
        }
        _ => format!("%{:02X}", b),
    }).collect()
}

// Tire compounds in F1.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub enum TireCompound {
    Soft,
    Medium,
    Hard,
    Intermediate,
    Wet,
}

// Track conditions.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub enum TrackStatus {
    Green,
    Yellow,
    SafetyCar,
    VirtualSafetyCar,
    RedFlag,
}

// Track parameters.
#[derive(Deserialize, Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct TrackParameters {
    pub total_laps: u32,
    pub base_pit_stop_loss_seconds: f64,
}

// Ego car state.
#[derive(Deserialize, Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct EgoCar {
    pub current_lap: u32,
    pub current_tire: TireCompound,
    pub tire_age_laps: u32,
    pub mandatory_pit_completed: bool,
    pub front_wing_damage: bool,
    pub time_penalty_seconds: f64,
}

// Competitor car state.
#[derive(Deserialize, Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct CompetitorCar {
    pub driver_name: String,
    pub team: String,
    pub current_lap: u32,
    pub gap_to_ego_seconds: f64,
    pub current_tire: TireCompound,
    pub tire_age_laps: u32,
    pub projected_pit_lap: u32,
}

// Race state.
#[derive(Deserialize, Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct RaceState {
    pub track: TrackParameters,
    pub environment: TrackStatus,
    pub ego_car: EgoCar,
    #[serde(default)]
    pub competitors: Vec<CompetitorCar>,
}

// Single planned pit stop.
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct PitStopPlan {
    pub pit_lap: u32,
    pub new_compound: TireCompound,
    pub pit_loss_seconds: f64,
    pub reasons: Vec<String>,
}

// Complete race strategy option.
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct StrategyOption {
    pub name: String,
    pub is_valid_f1_rules: bool,
    pub total_pit_stops: u32,
    pub pit_stops: Vec<PitStopPlan>,
    pub projected_total_time_seconds: f64,
    pub projected_total_time_formatted: String,
    pub average_lap_time_seconds: f64,
    pub total_degradation_loss_seconds: f64,
    pub total_pit_stop_time_loss_seconds: f64,
    pub delta_to_optimal_seconds: f64,
}

// Undercut / Overcut Option
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct UndercutOvercutOption {
    pub competitor_name: String,
    pub target_pit_lap: u32,
    pub pit_lap_delta: i32,
    pub ego_pit_lap: u32,
    pub advantage_seconds: f64,
}

// Strategy response.
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct MultiStrategyResponse {
    pub optimal_strategy: StrategyOption,
    pub alternative_strategies: Vec<StrategyOption>,
    pub safety_car_advantage_active: bool,
    pub summary_message: String,
    #[serde(default)]
    pub undercut_overcut_options: Vec<UndercutOvercutOption>,
    #[serde(default)]
    pub traffic_windows: Vec<TrafficWindowResponse>,
}

// Traffic Window Option
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct TrafficWindowResponse {
    pub pit_lap: u32,
    pub is_clean_air: bool,
    pub traffic_warning: String,
}

fn format_duration(seconds: f64) -> String {
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

fn get_compound_lifecycle(compound: &TireCompound) -> u32 {
    match compound {
        TireCompound::Soft => 20,
        TireCompound::Medium => 35,
        TireCompound::Hard => 50,
        TireCompound::Intermediate => 25,
        TireCompound::Wet => 40,
    }
}

fn get_degradation_factor(compound: &TireCompound, age: u32) -> f64 {
    let base_rate = match compound {
        TireCompound::Soft => 0.15,
        TireCompound::Medium => 0.08,
        TireCompound::Hard => 0.04,
        TireCompound::Intermediate => 0.25,
        TireCompound::Wet => 0.30,
    };

    let lifecycle = get_compound_lifecycle(compound) as f64;
    let cliff_lap = lifecycle * 0.8;
    
    let mut deg = base_rate * (age as f64);
    
    if (age as f64) > cliff_lap {
        let over = (age as f64) - cliff_lap;
        deg += (over * 0.2).exp() - 1.0; 
    }
    
    deg
}

fn calculate_pit_stop_loss(
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
                reasons.push(format!("Safety Car cheap pit stop discount (-{:.1}s)", base_loss * 0.40));
            }
            TrackStatus::VirtualSafetyCar => {
                pit_loss *= 0.70;
                reasons.push(format!("VSC cheap pit stop discount (-{:.1}s)", base_loss * 0.30));
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
        reasons.push(format!("Served time penalty (+{:.1}s)", time_penalty_seconds));
    }

    (pit_loss, reasons)
}

fn simulate_stint(
    start_lap: u32,
    end_lap: u32,
    compound: &TireCompound,
    initial_tire_age: u32,
    environment: &TrackStatus,
    competitors: &[CompetitorCar],
) -> (f64, f64) {
    if start_lap >= end_lap {
        return (0.0, 0.0);
    }

    let mut total_stint_time = 0.0;
    let mut total_deg_loss = 0.0;
    let mut age = initial_tire_age;

    for lap in start_lap..end_lap {
        let deg = get_degradation_factor(compound, age);
        let fuel_effect = (lap as f64) * -0.03;
        let track_evolution = (lap as f64) * -0.015;
        
        let mut dirty_air_penalty = 0.0;
        for comp in competitors {
            if comp.gap_to_ego_seconds > 0.0 && comp.gap_to_ego_seconds <= 1.5 {
                if lap < comp.projected_pit_lap {
                    dirty_air_penalty = 1.0;
                    break;
                }
            }
        }
        
        let lap_time = match environment {
            TrackStatus::RedFlag => 0.0,
            TrackStatus::SafetyCar | TrackStatus::VirtualSafetyCar => 90.0 + deg + 30.0 + fuel_effect + track_evolution + dirty_air_penalty,
            TrackStatus::Green | TrackStatus::Yellow => 90.0 + deg + fuel_effect + track_evolution + dirty_air_penalty,
        };

        if *environment != TrackStatus::RedFlag {
            total_deg_loss += deg;
        }

        total_stint_time += lap_time;
        age += 1;
    }

    (total_stint_time, total_deg_loss)
}

pub fn optimize_race_strategies(state: &RaceState) -> MultiStrategyResponse {
    let current_lap = state.ego_car.current_lap;
    let total_laps = state.track.total_laps;
    let laps_remaining = total_laps.saturating_sub(current_lap);

    let sc_active = matches!(state.environment, TrackStatus::SafetyCar | TrackStatus::VirtualSafetyCar);

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
        };

        return MultiStrategyResponse {
            optimal_strategy: dummy.clone(),
            alternative_strategies: vec![],
            safety_car_advantage_active: sc_active,
            summary_message: "Race completed. No laps remaining to simulate.".to_string(),
            undercut_overcut_options: vec![],
            traffic_windows: vec![],
        };
    }

    let candidate_compounds = match state.ego_car.current_tire {
        TireCompound::Intermediate | TireCompound::Wet => vec![TireCompound::Intermediate, TireCompound::Wet, TireCompound::Medium],
        _ => vec![TireCompound::Soft, TireCompound::Medium, TireCompound::Hard],
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
    );

    let is_valid_0stop = state.ego_car.mandatory_pit_completed;

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
            );

            let (stint2_time, stint2_deg) = simulate_stint(
                pit_lap,
                total_laps,
                new_compound,
                0,
                &state.environment,
                &state.competitors,
            );

            let total_time = stint1_time + pit_loss + stint2_time;
            let total_deg = stint1_deg + stint2_deg;
            let is_valid = state.ego_car.mandatory_pit_completed || *new_compound != state.ego_car.current_tire;

            let pit_plan = PitStopPlan {
                pit_lap,
                new_compound: new_compound.clone(),
                pit_loss_seconds: (pit_loss * 100.0).round() / 100.0,
                reasons: pit_reasons.clone(),
            };

            let name_extra = if is_pit_current_lap && sc_active { " (SC Box Now!)" } else { "" };

            evaluated_strategies.push(StrategyOption {
                name: format!("1-Stop: {:?} (L{}) ➔ {:?}{}", state.ego_car.current_tire, pit_lap, new_compound, name_extra),
                is_valid_f1_rules: is_valid,
                total_pit_stops: 1,
                pit_stops: vec![pit_plan],
                projected_total_time_seconds: total_time,
                projected_total_time_formatted: format_duration(total_time),
                average_lap_time_seconds: (total_time / laps_remaining as f64 * 100.0).round() / 100.0,
                total_degradation_loss_seconds: (total_deg * 100.0).round() / 100.0,
                total_pit_stop_time_loss_seconds: (pit_loss * 100.0).round() / 100.0,
                delta_to_optimal_seconds: 0.0,
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
                    let (s1_t, s1_d) = simulate_stint(current_lap, p1_lap, &state.ego_car.current_tire, state.ego_car.tire_age_laps, &state.environment, &state.competitors);
                    let (s2_t, s2_d) = simulate_stint(p1_lap, p2_lap, c1, 0, &state.environment, &state.competitors);
                    let (s3_t, s3_d) = simulate_stint(p2_lap, total_laps, c2, 0, &state.environment, &state.competitors);

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

                    evaluated_strategies.push(StrategyOption {
                        name: format!("2-Stop: {:?} (L{}) ➔ {:?} (L{}) ➔ {:?}", state.ego_car.current_tire, p1_lap, c1, p2_lap, c2),
                        is_valid_f1_rules: true,
                        total_pit_stops: 2,
                        pit_stops: vec![pit1, pit2],
                        projected_total_time_seconds: total_time,
                        projected_total_time_formatted: format_duration(total_time),
                        average_lap_time_seconds: (total_time / laps_remaining as f64 * 100.0).round() / 100.0,
                        total_degradation_loss_seconds: (total_deg * 100.0).round() / 100.0,
                        total_pit_stop_time_loss_seconds: (total_pit_loss * 100.0).round() / 100.0,
                        delta_to_optimal_seconds: 0.0,
                    });
                }
            }
        }
    }

    evaluated_strategies.sort_by(|a, b| {
        match (a.is_valid_f1_rules, b.is_valid_f1_rules) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.projected_total_time_seconds
                .partial_cmp(&b.projected_total_time_seconds)
                .unwrap_or(std::cmp::Ordering::Equal),
        }
    });

    let optimal = evaluated_strategies[0].clone();
    let min_time = optimal.projected_total_time_seconds;

    let mut alternatives: Vec<StrategyOption> = Vec::new();
    for strat in evaluated_strategies.into_iter().skip(1).take(5) {
        let mut alt = strat;
        alt.delta_to_optimal_seconds = ((alt.projected_total_time_seconds - min_time) * 100.0).round() / 100.0;
        alternatives.push(alt);
    }

    let summary = format!(
        "Optimal Strategy: '{}' with projected time of {}. (Simulated {} laps remaining).",
        optimal.name,
        optimal.projected_total_time_formatted,
        laps_remaining
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

        let baseline_s1 = simulate_stint(current_lap, target_lap, &state.ego_car.current_tire, state.ego_car.tire_age_laps, &state.environment, &state.competitors).0;
        let baseline_s2 = simulate_stint(target_lap, total_laps, &candidate_new_compound, 0, &state.environment, &state.competitors).0;
        let baseline_total = baseline_s1 + baseline_s2;

        for delta in [-3, -2, -1, 1, 2, 3] {
            let ego_pit_lap = (target_lap as i32 + delta) as u32;
            if ego_pit_lap > current_lap && ego_pit_lap < total_laps {
                let s1 = simulate_stint(current_lap, ego_pit_lap, &state.ego_car.current_tire, state.ego_car.tire_age_laps, &state.environment, &state.competitors).0;
                let s2 = simulate_stint(ego_pit_lap, total_laps, &candidate_new_compound, 0, &state.environment, &state.competitors).0;
                let total = s1 + s2;
                
                let advantage = baseline_total - total; // positive means faster

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
            // Very rough estimate of gap after pit
            let comp_loss = if comp.projected_pit_lap < pit_lap { pit_loss } else { 0.0 };
            let projected_gap = comp.gap_to_ego_seconds + pit_loss - comp_loss;
            
            // If emerging within 2.5 seconds of a competitor, risk of DRS train
            if projected_gap.abs() < 2.5 {
                clean_air = false;
                warning = format!("Traffic warning: will emerge into DRS train with {}", comp.driver_name);
                break;
            }
        }
        
        traffic_windows.push(TrafficWindowResponse {
            pit_lap,
            is_clean_air: clean_air,
            traffic_warning: warning,
        });
    }

    MultiStrategyResponse {
        optimal_strategy: optimal,
        alternative_strategies: alternatives,
        safety_car_advantage_active: sc_active,
        summary_message: summary,
        undercut_overcut_options,
        traffic_windows,
    }
}

/// Simulate a race strategy for a given race state.
#[utoipa::path(
    post,
    path = "/simulate",
    request_body = RaceState,
    responses(
        (status = 200, description = "Multi-strategy simulation result", body = MultiStrategyResponse)
    )
)]
async fn simulate_race(Json(payload): Json<RaceState>) -> Json<MultiStrategyResponse> {
    let response = optimize_race_strategies(&payload);
    Json(response)
}

#[derive(OpenApi)]
#[openapi(
    paths(simulate_race, get_tracks, get_circuit_geometry),
    components(schemas(RaceState, TrackParameters, TrackStatus, EgoCar, CompetitorCar, TireCompound, PitStopPlan, StrategyOption, UndercutOvercutOption, TrafficWindowResponse, MultiStrategyResponse, CircuitInfo)),
    tags(
        (name = "F1 Strategy Engine", description = "API for simulating multi-stop race strategies")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/tracks", get(get_tracks))
        .route("/simulate", post(simulate_race))
        .route("/api/circuit-geometry", get(get_circuit_geometry));

    let app = app.merge(
        SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi())
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🏎️ Strategy Engine running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulate_stint() {
        let (time, deg) = simulate_stint(1, 10, &TireCompound::Soft, 0, &TrackStatus::Green, &[]);
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
        let (loss, _) = calculate_pit_stop_loss(20.0, &TrackStatus::VirtualSafetyCar, false, 0.0, true);
        assert_eq!(loss, 14.0); // 20.0 * 0.7
    }

    #[test]
    fn test_optimize_race_strategies_valid_0_stop() {
        let state = RaceState {
            track: TrackParameters {
                total_laps: 50,
                base_pit_stop_loss_seconds: 20.0,
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
        };
        let response = optimize_race_strategies(&state);
        // Best strategy might be 0 stop or 1 stop, but 0 stop MUST be valid
        assert!(response.optimal_strategy.is_valid_f1_rules);
    }

    #[test]
    fn test_optimize_race_strategies_invalid_compound() {
        let state = RaceState {
            track: TrackParameters {
                total_laps: 50,
                base_pit_stop_loss_seconds: 20.0,
            },
            environment: TrackStatus::Green,
            ego_car: EgoCar {
                current_lap: 45,
                current_tire: TireCompound::Hard,
                tire_age_laps: 5,
                mandatory_pit_completed: false, // Did not do mandatory pit!
                front_wing_damage: false,
                time_penalty_seconds: 0.0,
            },
            competitors: vec![],
        };
        let response = optimize_race_strategies(&state);
        // The optimal strategy should have 1 stop, because 0 stop is invalid
        // (optimal strategies are sorted to put valid ones first)
        assert!(response.optimal_strategy.is_valid_f1_rules);
        assert!(response.optimal_strategy.total_pit_stops > 0);
    }
}