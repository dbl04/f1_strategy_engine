use axum::{
    routing::{get, post},
    response::Html,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Serve the interactive F1 Strategy Engine Dashboard UI at GET /
async fn index_handler() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

// This enum defines the possible tire compounds in F1.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub enum TireCompound {
    Soft,
    Medium,
    Hard,
    Intermediate,
    Wet,
}

// This enum defines the possible track conditions.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub enum TrackStatus {
    Green,
    Yellow,
    SafetyCar,
    VirtualSafetyCar,
    RedFlag,
}

// Parameters of a track, such as total laps and standard pit stop time loss.
#[derive(Deserialize, Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct TrackParameters {
    pub total_laps: u32,
    pub base_pit_stop_loss_seconds: f64,
}

// State of the ego car, including lap count, tire condition, damage, and mandatory pit status.
#[derive(Deserialize, Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct EgoCar {
    pub current_lap: u32,
    pub current_tire: TireCompound,
    pub tire_age_laps: u32,
    pub mandatory_pit_completed: bool,
    pub front_wing_damage: bool,
    pub time_penalty_seconds: f64,
}

// Overall state of the race environment and car.
#[derive(Deserialize, Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct RaceState {
    pub track: TrackParameters,
    pub environment: TrackStatus,
    pub ego_car: EgoCar,
}

// Details of a single planned pit stop.
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct PitStopPlan {
    pub pit_lap: u32,
    pub new_compound: TireCompound,
    pub pit_loss_seconds: f64,
    pub reasons: Vec<String>,
}

// Details of a complete race strategy option (0-stop, 1-stop, 2-stop).
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

// Response returned by the multi-strategy optimization engine.
#[derive(Serialize, Debug, Clone, utoipa::ToSchema)]
pub struct MultiStrategyResponse {
    pub optimal_strategy: StrategyOption,
    pub alternative_strategies: Vec<StrategyOption>,
    pub safety_car_advantage_active: bool,
    pub summary_message: String,
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

fn get_degradation_factor(compound: &TireCompound) -> f64 {
    match compound {
        TireCompound::Soft => 0.15,
        TireCompound::Medium => 0.08,
        TireCompound::Hard => 0.04,
        TireCompound::Intermediate => 0.25,
        TireCompound::Wet => 0.30,
    }
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
                pit_loss *= 0.60; // 40% discount under SC
                reasons.push(format!("Safety Car cheap pit stop discount (-{:.1}s)", base_loss * 0.40));
            }
            TrackStatus::VirtualSafetyCar => {
                pit_loss *= 0.70; // 30% discount under VSC
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
) -> (f64, f64) {
    if start_lap >= end_lap {
        return (0.0, 0.0);
    }

    let deg_factor = get_degradation_factor(compound);
    let mut total_stint_time = 0.0;
    let mut total_deg_loss = 0.0;
    let mut age = initial_tire_age;

    for _ in start_lap..end_lap {
        let deg = deg_factor * age as f64;
        let lap_time = match environment {
            TrackStatus::RedFlag => 0.0,
            TrackStatus::SafetyCar | TrackStatus::VirtualSafetyCar => 90.0 + deg + 30.0,
            TrackStatus::Green | TrackStatus::Yellow => 90.0 + deg,
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
        };
    }

    let candidate_compounds = match state.ego_car.current_tire {
        TireCompound::Intermediate | TireCompound::Wet => vec![TireCompound::Intermediate, TireCompound::Wet, TireCompound::Medium],
        _ => vec![TireCompound::Soft, TireCompound::Medium, TireCompound::Hard],
    };

    let mut evaluated_strategies: Vec<StrategyOption> = Vec::new();

    // 1. Evaluate 0-Stop Strategy
    let (time_0stop, deg_0stop) = simulate_stint(
        current_lap,
        total_laps,
        &state.ego_car.current_tire,
        state.ego_car.tire_age_laps,
        &state.environment,
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

    // 2. Evaluate 1-Stop Strategies
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
            );

            let (stint2_time, stint2_deg) = simulate_stint(
                pit_lap,
                total_laps,
                new_compound,
                0,
                &state.environment,
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

    // 3. Evaluate 2-Stop Strategies
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
                    let (s1_t, s1_d) = simulate_stint(current_lap, p1_lap, &state.ego_car.current_tire, state.ego_car.tire_age_laps, &state.environment);
                    let (s2_t, s2_d) = simulate_stint(p1_lap, p2_lap, c1, 0, &state.environment);
                    let (s3_t, s3_d) = simulate_stint(p2_lap, total_laps, c2, 0, &state.environment);

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

    // Sort strategies: valid strategies first, then lowest total projected time
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

    MultiStrategyResponse {
        optimal_strategy: optimal,
        alternative_strategies: alternatives,
        safety_car_advantage_active: sc_active,
        summary_message: summary,
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
    paths(simulate_race),
    components(schemas(RaceState, TrackParameters, TrackStatus, EgoCar, TireCompound, PitStopPlan, StrategyOption, MultiStrategyResponse)),
    tags(
        (name = "F1 Strategy Engine", description = "API for simulating multi-stop race strategies")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/simulate", post(simulate_race));

    let app = app.merge(
        SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi())
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("🏎️ Strategy Engine running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}