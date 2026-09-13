# Autonomous Agent Execution Prompt - F1 Strategy Engine Iteration Loop

You are an autonomous AI coding assistant running an iterative development loop on the **F1 Strategy Engine** project. Your goal is to systematically implement, refine, and verify features defined in [PRD.md](file:///c:/Users/Daniel/Desktop/Projects/f1_strategy_engine/PRD.md).

---

## 1. Loop Objective & Execution Strategy

Execute requirements from `PRD.md` in incremental steps. For each iteration in the loop:
1. **Identify Next Phase / Sub-task**: Check current implementation state in `src/main.rs`, `src/index.html`, and `Cargo.toml`.
2. **Implement Code**: Make targeted, minimal, robust changes in Rust backend or HTML/JS frontend.
3. **Verify Implementation**:
   - Run `cargo check` to verify zero compilation errors or warnings.
   - Run `cargo test` to execute all unit tests.
   - Validate API routes and responses where appropriate.
4. **Refine & Document**: Ensure clean comments, correct docstrings, and update Swagger OpenAPI schema components (`utoipa`).

---

## 2. Iterative Task Breakdown

### Phase 1: Core Physics & Degradation Engine Refinement
- [ ] **Non-Linear Tyre Degradation**: Update `get_degradation_factor` and `simulate_stint` to include non-linear exponential degradation when tire age exceeds 80% of compound lifecycle.
- [ ] **Fuel Load Weight Effect**: Add dynamic fuel weight consumption model (~1.8kg/lap fuel burn rate, ~0.03s lap time reduction per lap).
- [ ] **Track Evolution Rubbering**: Add track rubbering factor (lap time improves by up to ~0.015s per lap as track rubbers in).
- [ ] **Unit Tests**: Write comprehensive Rust unit tests in `src/main.rs` testing stint calculations, 0-stop/1-stop/2-stop evaluations, penalty additions, and FIA compound validity checks.

### Phase 2: Competitor Traffic & Undercut / Overcut Gap Simulator
- [ ] **Traffic Model Data Structures**: Introduce `CompetitorCar` struct (`driver_name`, `team`, `current_lap`, `gap_to_ego_seconds`, `current_tire`, `tire_age_laps`, `projected_pit_lap`).
- [ ] **Dirty Air Time Loss**: Implement dirty air time penalty logic (+1.0s/lap penalty when ego car is within 1.5s gap behind a rival).
- [ ] **Undercut / Overcut Calculator**: Calculate delta advantage gained by pitting 1 to 3 laps earlier vs later than target competitor car.
- [ ] **Pit Exit Traffic Window Release**: Identify clean air gaps on track after pit exit to avoid boxing into a DRS train.

### Phase 3: Dynamic Weather & Safety Car Probability Module
- [ ] **Dynamic Weather Transitions**: Support weather forecast track state changes (Dry $\rightarrow$ Damp $\rightarrow$ Wet) mid-race with optimal crossover lap calculation (e.g. Slick to Intermediate crossover delta).
- [ ] **Monte Carlo SC Risk Assessor**: Calculate real-time Safety Car probability per lap based on track historical SC deployment rates (e.g. Monaco 80%, Monza 25%).
- [ ] **Virtual Safety Car Advantage Engine**: Evaluate optimal "VSC Window" strategy options, highlighting instant box calls when VSC is active.

### Phase 4: Broadcast Telemetry Dashboard UI Enhancements
- [ ] **Leaflet Circuit 2D Vector Map Integration**: Render real 2D track layout coordinates fetched from `/api/circuit-geometry` proxy onto Leaflet.js map with animated car markers.
- [ ] **Dynamic Strategy Matrix & Stint Visualizer**: Render horizontal color-coded stint bars (Red=Soft, Yellow=Medium, White=Hard, Green=Inter, Blue=Wet) showing exact pit laps and compound stint lengths.
- [ ] **Competitor Traffic Radar Visualizer**: Display relative gaps to leader and rival cars in an interactive timing tower display.

### Phase 5: REST API Extension & OpenAPI Docs
- [ ] **Expanded OpenAPI Specifications**: Ensure all new structs (`CompetitorCar`, `WeatherState`, `TrafficWindowResponse`) are annotated with `#[derive(utoipa::ToSchema)]` and registered in `ApiDoc`.
- [ ] **Swagger UI Validation**: Verify `/docs` endpoint accurately lists all paths and request/response schemas.

---

## 3. Strict Coding Rules & Guardrails

- **Zero Panic Safety**: Do NOT use `.unwrap()` or `.expect()` on untrusted input or HTTP requests in Axum handler endpoints. Always handle errors using Rust `Result` or fallback defaults.
- **API Signature Integrity**: When updating data structures (`RaceState`, `StrategyOption`), ensure all call sites, API endpoints, serialization logic (`serde`), and frontend JavaScript fetches in `src/index.html` remain consistent.
- **F1 UI Styling Guidelines**:
  - Always maintain official F1 dark broadcast aesthetics (`#15151e`, `#e10600`, high-contrast typography).
  - Use smooth transitions and readable telemetry typography.
- **Verification Command Requirement**:
  - After making code changes, ALWAYS run:
    ```powershell
    cargo check
    cargo test
    ```
  - Fix any warnings or build failures immediately before proceeding to the next task.
