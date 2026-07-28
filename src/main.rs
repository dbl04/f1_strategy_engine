use axum::{
    routing::post,
    Router,
    Json,
};
use serde::{Deserialize, Serialize};

// 1. Define the incoming JSON request
#[derive(Deserialize, Debug)]
pub struct RaceStrategyRequest {
    pub total_laps: u32,
    pub track_name: String,
}

// 2. Define the outgoing JSON response
#[derive(Serialize)]
pub struct RaceStrategyResponse {
    pub estimated_time_seconds: f64,
    pub message: String,
}

// 3. The Route Handler (Our pit wall)
async fn simulate_race(Json(payload): Json<RaceStrategyRequest>) -> Json<RaceStrategyResponse> {
    println!("Received strategy request for {}", payload.track_name);
    
    // We will build the actual tire degradation math here next
    let response = RaceStrategyResponse {
        estimated_time_seconds: 5400.0, // Hardcoded for now
        message: format!("Simulated {} laps", payload.total_laps),
    };
    
    Json(response)
}

// 4. The Server Engine
#[tokio::main]
async fn main() {
    let app = Router::new().route("/simulate", post(simulate_race));
    
    // Bind the server to port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    
    println!("🏎️ Strategy Engine running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}