# Product Requirements Document (PRD) - F1 Strategy Engine

## 1. Executive Summary & Vision

The **F1 Strategy Engine** is a high-performance, real-time Formula 1 pit strategy simulation and telemetry optimization platform built in Rust. It provides race engineers, strategists, and simulation enthusiasts with instant strategy decision support, calculating optimal pit stop windows, multi-stint compound selections, undercut/overcut probabilities, and Safety Car / VSC advantage windows.

The platform combines a lightweight, high-throughput Rust backend (`Axum` + `Tokio`) with interactive OpenAPI documentation (`Utoipa` Swagger UI) and an official F1 TV Broadcast-style web dashboard (`index.html`) featuring real track geometry fetched live via OpenStreetMap Overpass API proxies.

---

## 2. Core Architecture & System Topology

```
                  +----------------------------------+
                  |    F1 TV Broadcast Web UI        |
                  | (index.html - HTML5/CSS3/JS/Leaflet)|
                  +-----------------+----------------+
                                    | HTTP / JSON
                                    v
+-------------------------------------------------------------------+
|                        Rust Axum Web Server                       |
|  +---------------------+   +----------------------------------+   |
|  | OpenAPI / Swagger   |   | Strategy Optimization Engine     |   |
|  | (/docs, /api-docs)  |   | (Stint Simulator & Algorithm)    |   |
|  +---------------------+   +----------------------------------+   |
|  +---------------------+   +----------------------------------+   |
|  | Circuits & Track    |   | Overpass API Proxy               |   |
|  | Metadata (/api/tracks) |   | (/api/circuit-geometry)          |   |
|  +---------------------+   +----------------------------------+   |
+-----------------------------------+-------------------------------+
                                    |
                                    v
                  +----------------------------------+
                  | OpenStreetMap Overpass API       |
                  | (Real 2D Circuit Coordinates)    |
                  +----------------------------------+
```

---

## 3. Detailed Functional Requirements

### 3.1 Strategy Optimization & Simulation Engine
- **Multi-Stint Strategy Evaluation**:
  - Automatically evaluate **0-Stop**, **1-Stop**, **2-Stop**, and **3-Stop** strategy variants for any given race state.
  - Model lap-by-lap pace degradation based on tire compound choice (Soft, Medium, Hard, Intermediate, Wet) and tire age.
- **F1 Sporting Regulations Compliance**:
  - Enforce mandatory dry compound changes (e.g. driver must run at least two different dry compounds in a dry race unless rain occurs or mandatory pit is already fulfilled).
  - Flag strategies as valid or invalid based on FIA rules.
- **Dynamic Safety Car (SC) & Virtual Safety Car (VSC) Discounts**:
  - Apply time-loss discounts for pit stops executed during active SC (~40% time savings) or VSC (~30% time savings).
  - Provide immediate `"SC Box Now!"` callout indicators when pitting on current lap yields maximum delta advantage.
- **Damage & Penalty Adjustments**:
  - Incorporate front wing replacement time penalties (+10.0s) and accrued time penalties (e.g. 5s/10s penalty served in pit box).

### 3.2 Physics & Degradation Models
1. **Tire Degradation Formula**:
   $$\text{Degradation Loss (seconds/lap)} = \text{Factor}_{\text{compound}} \times \text{Tire Age (laps)}$$
   - Soft: $0.15\text{s/lap}$ degradation rate
   - Medium: $0.08\text{s/lap}$ degradation rate
   - Hard: $0.04\text{s/lap}$ degradation rate
   - Intermediate: $0.25\text{s/lap}$ degradation rate
   - Wet: $0.30\text{s/lap}$ degradation rate
2. **Pace Delta Model**:
   - Base lap pace: $90.0\text{s}$ (standard Green Flag conditions).
   - SC/VSC delta pace: $120.0\text{s}$ per lap under neutralised conditions.
3. **Fuel Weight Decay Model**:
   - Fuel burn rate: ~1.8kg/lap, reducing lap time by ~0.03s per lap burned.

### 3.3 Competitor Traffic & Undercut/Overcut Matrix (Phase 2 Upgrade)
- Model competitor pit windows and relative track positioning.
- Calculate dirty air penalties (+0.8s to +1.5s/lap when within 1.5s of a rival car).
- Identify clean air pit exit release slots.

### 3.4 Circuit Metadata & Overpass API Vector Proxy
- **Supported Tracks**: Monza, Silverstone, Spa-Francorchamps, Monaco, Bahrain, Suzuka, COTA, Singapore, and expandable catalog.
- **Overpass Proxy Endpoint (`/api/circuit-geometry`)**:
  - Server-side HTTP proxy fetching real geographic lat/lng vectors from OpenStreetMap.
  - Prevents CORS issues in client-side Leaflet renderings.

### 3.5 F1 TV Broadcast Frontend UI
- Official telemetry styling featuring high-contrast dark theme (`#15151e`), official F1 brand red (`#e10600`), and skewed broadcast typography (`Titillium Web` / `Space Grotesk`).
- Interactive telemetry controls: Track selector, current lap slider, tire selector, track status toggle (Green, Yellow, SC, VSC, Red Flag), damage/penalty toggles.
- Real-time Leaderboard & Strategy Comparison Table displaying delta to optimal, projected finish time, stint breakdown, and pit stop reasons.

---

## 4. API Specification & Schemas

### 4.1 Endpoints
- `GET /`: Serves static web UI dashboard (`index.html`).
- `GET /api/tracks`: Returns list of official circuit metadata.
- `POST /simulate`: Executes strategy simulation for a given `RaceState`.
- `GET /api/circuit-geometry?name={circuit_name}`: Proxies vector geometry from Overpass API.
- `GET /docs`: Serves Swagger UI documentation for OpenAPI specs.

### 4.2 Core Data Schemas (`Rust`)
```rust
pub enum TireCompound { Soft, Medium, Hard, Intermediate, Wet }
pub enum TrackStatus { Green, Yellow, SafetyCar, VirtualSafetyCar, RedFlag }

pub struct TrackParameters {
    pub total_laps: u32,
    pub base_pit_stop_loss_seconds: f64,
}

pub struct EgoCar {
    pub current_lap: u32,
    pub current_tire: TireCompound,
    pub tire_age_laps: u32,
    pub mandatory_pit_completed: bool,
    pub front_wing_damage: bool,
    pub time_penalty_seconds: f64,
}

pub struct RaceState {
    pub track: TrackParameters,
    pub environment: TrackStatus,
    pub ego_car: EgoCar,
}
```

---

## 5. Non-Functional & Quality Requirements

1. **Performance**: Strategy evaluation under 10ms for up to 150 candidate combinations.
2. **Reliability**: Zero panics (`unwrap()` safety in core handler logic).
3. **Usability**: Fully responsive, high-framerate interactive dashboard working smoothly across modern web browsers.
4. **Code Quality**: Rust 2024 edition standard compliance, clean modularization, and unit test coverage across all simulation helper functions.
