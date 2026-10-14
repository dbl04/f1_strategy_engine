use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use f1_strategy_engine::api::create_router;
use f1_strategy_engine::domain::{
    CircuitInfo, EgoCar, MultiStrategyResponse, RaceState, TireCompound, TrackParameters,
    TrackStatus,
};
use tower::ServiceExt;

#[tokio::test]
async fn test_get_tracks_endpoint() {
    let app = create_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tracks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let tracks: Vec<CircuitInfo> = serde_json::from_slice(&body).unwrap();
    assert!(!tracks.is_empty());
}

#[tokio::test]
async fn test_simulate_endpoint() {
    let app = create_router();

    let state = RaceState {
        track: TrackParameters {
            total_laps: 50,
            base_pit_stop_loss_seconds: 20.0,
            historical_sc_probability: 0.3,
        },
        environment: TrackStatus::Green,
        ego_car: EgoCar {
            current_lap: 10,
            current_tire: TireCompound::Medium,
            tire_age_laps: 10,
            mandatory_pit_completed: false,
            front_wing_damage: false,
            time_penalty_seconds: 0.0,
        },
        competitors: vec![],
        weather_forecast: None,
    };

    let payload = serde_json::to_string(&state).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulate")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: MultiStrategyResponse = serde_json::from_slice(&body).unwrap();
    assert!(result.optimal_strategy.is_valid_f1_rules);
    assert!(result.optimal_strategy.monte_carlo_metrics.is_some());
}

#[tokio::test]
async fn test_circuit_geometry_by_id_endpoint() {
    let app = create_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/circuit-geometry/monza")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(!body.is_empty());
}

#[tokio::test]
async fn test_baked_track_endpoint() {
    let app = create_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/track/9161")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(!body.is_empty());
}

#[tokio::test]
async fn test_simulate_multi_endpoint() {
    let app = create_router();

    let req = f1_strategy_engine::engine::multi_driver::MultiDriverSimRequest {
        track_id: "silverstone".to_string(),
        total_laps: Some(52),
        current_lap: Some(25),
        pit_loss_seconds: Some(20.5),
        drivers: vec![],
    };

    let payload = serde_json::to_string(&req).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/simulate-multi")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: f1_strategy_engine::engine::multi_driver::MultiDriverSimResponse =
        serde_json::from_slice(&body).unwrap();
    assert_eq!(result.results.len(), 5);
    assert!(!result.winner_driver_id.is_empty());
}

#[tokio::test]
async fn test_live_state_endpoint() {
    let app = create_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/live-state/9161")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: f1_strategy_engine::api::openf1::LiveStateResponse =
        serde_json::from_slice(&body).unwrap();
    assert!(!result.drivers.is_empty());
    assert!(result.dynamic_odi.is_some());
    // Check timing tower fields on driver 1
    let first = &result.drivers[0];
    assert!(!first.compound.is_empty());
    assert!(first.current_lap > 0);
}

#[tokio::test]
async fn test_simulate_driver_strategy_endpoint() {
    let app = create_router();

    let req = f1_strategy_engine::engine::driver_sim::DriverSimulationRequest {
        session_key: 9000,
        driver_number: 1,
        pit_lap: 22,
        new_compound: f1_strategy_engine::domain::TireCompound::Hard,
        current_lap: 18,
        track_id: Some("monza".to_string()),
        target_rival_number: Some(3),
    };

    let payload = serde_json::to_string(&req).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/strategy/simulate-driver")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: f1_strategy_engine::engine::driver_sim::DriverSimulationResponse =
        serde_json::from_slice(&body).unwrap();
    assert_eq!(result.driver_tla, "NOR");
    assert_eq!(result.teammate_tla, "PIA");
    assert!(result.projected_finish_position >= 1);
    assert!(!result.traffic_risk.is_empty());
    assert!(result.monte_carlo_metrics.win_probability_pct > 0.0);
    assert!(!result.lap_timeline.is_empty());
}



