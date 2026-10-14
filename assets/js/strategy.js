// ==========================================
// 5. STRATEGY ENGINE & PIT WALL SIMULATION CONTROLLER
// ==========================================

let selectedTargetCompound = "Hard";
let currentTrackStatus = "Green"; // "Green" | "SafetyCar" | "VirtualSafetyCar" | "RedFlag"

// Simulated Trajectory Overlay State & Vehicle Dynamics Engine
const simOverlayState = {
  isActive: false,
  driverNumber: 1,
  driverTla: "NOR",
  ghostMode: true,
  isPlaying: true,
  playbackSpeed: 1,

  // Real Physics & Pit Stop State Machine
  status: "RACING", // "RACING" | "ENTERING_PITS" | "IN_PIT_BOX" | "EXITING_PITS"
  pitRequested: false, // Flag to execute pit stop on this lap
  currentLap: 20,
  newCompound: "Hard",
  currentCompound: "Medium",
  tyreAge: 15,
  pitStopCount: 0,
  simulatedProgress: 0.0,
  baseLapTime: 82.5,
  lastLapTime: 82.5,
  pitStopSecondsRemaining: 2.4,
  stationaryDuration: 2.4,
  isCurrentlyBoxing: false,

  // UI & Projections
  projectedFinishPos: 2,
  posDelta: 1,
  totalLaps: 53
};

function normalizeCompound(c) {
  if (!c) return "Medium";
  const s = String(c).trim().toUpperCase();
  if (s.startsWith("SOFT")) return "Soft";
  if (s.startsWith("HARD")) return "Hard";
  if (s.startsWith("INTER")) return "Intermediate";
  if (s.startsWith("WET")) return "Wet";
  return "Medium";
}

// Select compound via tactile pill buttons in the UI
function selectTargetCompound(comp) {
  selectedTargetCompound = normalizeCompound(comp);
  
  // Update active pill button UI
  document.querySelectorAll('.comp-pill').forEach(btn => {
    btn.classList.remove('active');
  });
  const activeBtn = document.querySelector(`.pill-${selectedTargetCompound.toLowerCase()}`);
  if (activeBtn) activeBtn.classList.add('active');

  // Update button subtitle description
  const sub = document.getElementById('btnBoxSubtitle');
  if (sub) {
    if (simOverlayState.pitRequested) {
      sub.innerText = `CONFIRMED: PITTING FOR ${selectedTargetCompound.toUpperCase()} TYRES`;
    } else {
      sub.innerText = `FIT NEW ${selectedTargetCompound.toUpperCase()} TYRES • SWITCH TO SIMULATION`;
    }
  }

  if (simOverlayState.isActive) {
    simOverlayState.newCompound = selectedTargetCompound;
  }
}

// Inward geometric offset for cars traveling through the pit lane
function getPitLaneCoords(progress, coords) {
  const cx = 0.5;
  const cy = 0.5;
  const dx = cx - coords.x;
  const dy = cy - coords.y;
  const dist = Math.hypot(dx, dy) || 1.0;
  return {
    x: coords.x + (dx / dist) * 0.025,
    y: coords.y + (dy / dist) * 0.025
  };
}

// Bulletproof, instant Pit Stop Trigger (Local execution with zero network dependency)
function triggerBoxThisLap() {
  const driverObj = lerpDriversMap.get(followedDriverNo) || Array.from(lerpDriversMap.values())[0];
  if (!driverObj) return;

  const curLap = driverObj.current_lap || 1;
  const curProg = driverObj.progress !== undefined ? driverObj.progress : 0.5;

  // 1. If not yet in simulation mode, branch driver off into digital twin immediately
  if (!simOverlayState.isActive || simOverlayState.driverNumber !== followedDriverNo) {
    simOverlayState.isActive = true;
    simOverlayState.driverNumber = followedDriverNo;
    simOverlayState.driverTla = driverObj.tla || getDriverTla(driverObj);
    simOverlayState.currentLap = curLap;
    simOverlayState.simulatedProgress = curProg;
    simOverlayState.currentCompound = driverObj.compound || "Medium";
    simOverlayState.tyreAge = driverObj.tyre_age || 0;
    simOverlayState.pitStopCount = driverObj.pit_stop_count || 0;
    simOverlayState.playbackSpeed = (typeof dvrState !== 'undefined' && dvrState.playbackSpeed) ? dvrState.playbackSpeed : 1;
    simOverlayState.isPlaying = (typeof dvrState !== 'undefined') ? dvrState.isPlaying : true;
  }

  // 2. Schedule pit stop for this lap to the user's chosen compound
  simOverlayState.newCompound = selectedTargetCompound;
  simOverlayState.pitRequested = true;
  simOverlayState.isCurrentlyBoxing = true;
  simOverlayState.pitStopSecondsRemaining = 2.4;

  // If car is already within pit entrance zone, transition directly into pits
  if (simOverlayState.simulatedProgress >= 0.88 && simOverlayState.simulatedProgress < 0.94) {
    simOverlayState.status = "ENTERING_PITS";
  }

  // 3. Update UI buttons and banners immediately
  const btnTitle = document.getElementById('btnBoxTitle');
  const btnSub = document.getElementById('btnBoxSubtitle');
  const btnBox = document.getElementById('btnBoxThisLap');
  const btnReset = document.getElementById('btnResetToLive');

  if (btnTitle) btnTitle.innerText = "🚨 BOX CONFIRMED";
  if (btnSub) btnSub.innerText = `PITTING THIS LAP ➔ ${selectedTargetCompound.toUpperCase()}`;
  if (btnBox) btnBox.classList.add('box-confirmed');
  if (btnReset) btnReset.classList.remove('hidden');

  const modeTag = document.getElementById('simModeTag');
  if (modeTag) {
    modeTag.className = "telemetry-mode-tag mode-sim";
    modeTag.innerText = "🟣 SIMULATED DIGITAL TWIN";
  }

  // Show Sim Banner above visualizer
  const banner = document.getElementById('simModeBanner');
  if (banner) banner.classList.remove('hidden');
  const dEl = document.getElementById('simBannerDriver');
  if (dEl) dEl.innerText = `#${simOverlayState.driverNumber} ${simOverlayState.driverTla}`;
  const pEl = document.getElementById('simBannerPlan');
  if (pEl) pEl.innerText = `(🚨 BOX THIS LAP ➔ ${selectedTargetCompound.toUpperCase()})`;

  // Immediately reflect pit flag in timing tower
  driverObj.isPitLap = true;
  renderTimingTowerTable();

  // 4. Asynchronously update strategy projections in background without blocking
  runSingleDriverSimulation(false, true);
}

// 60 FPS Vehicle Dynamics & Pit Stop Physical Simulator
function tickSimulatedDriver(deltaTime) {
  if (!simOverlayState.isActive || !simOverlayState.isPlaying) return;
  if (typeof dvrState !== 'undefined' && !dvrState.isPlaying) return;

  const activeSpeed = (typeof dvrState !== 'undefined' && dvrState.playbackSpeed)
    ? dvrState.playbackSpeed
    : (simOverlayState.playbackSpeed || 1);
  simOverlayState.playbackSpeed = activeSpeed;

  const dt = deltaTime * activeSpeed;
  if (dt <= 0) return;

  // 1. Neutralization & Pace Modifiers
  let speedModifier = 1.0;
  if (currentTrackStatus === "SafetyCar") {
    speedModifier = 0.60; // 40% reduction under SC delta pace
  } else if (currentTrackStatus === "VirtualSafetyCar") {
    speedModifier = 0.70; // 30% reduction under VSC delta pace
  } else if (currentTrackStatus === "RedFlag") {
    speedModifier = 0.0;  // Full race halt
  }

  // 2. Compound Pace Delta & Tyre Degradation
  let compoundDelta = 0.0;
  let degRate = 0.06;
  const comp = normalizeCompound(simOverlayState.currentCompound);
  if (comp === "Soft") {
    compoundDelta = -0.65;
    degRate = 0.11;
  } else if (comp === "Medium") {
    compoundDelta = 0.0;
    degRate = 0.06;
  } else if (comp === "Hard") {
    compoundDelta = 0.70;
    degRate = 0.035;
  } else if (comp === "Intermediate") {
    compoundDelta = 3.5;
    degRate = 0.08;
  } else if (comp === "Wet") {
    compoundDelta = 7.0;
    degRate = 0.08;
  }

  const effectiveLapTime = Math.max(60.0, (simOverlayState.baseLapTime + compoundDelta + (simOverlayState.tyreAge * degRate)) / (speedModifier > 0.01 ? speedModifier : 1.0));
  simOverlayState.lastLapTime = effectiveLapTime;

  // Normalized track progress rate per second (1.0 = full lap)
  const baseRate = (speedModifier > 0.01) ? (1.0 / effectiveLapTime) : 0.0;

  // 3. Physical Pit Stop Execution State Machine
  if (simOverlayState.pitRequested) {
    // A) Pit Entry Lane (approaching straight: ~0.88 to 0.93)
    if (simOverlayState.simulatedProgress >= 0.88 && simOverlayState.simulatedProgress < 0.93) {
      simOverlayState.status = "ENTERING_PITS";
      simOverlayState.simulatedProgress += baseRate * 0.25 * dt; // Pit lane speed limiter (80 km/h)
    }
    // B) Pit Box Area (~0.93 to 0.96)
    else if (simOverlayState.simulatedProgress >= 0.93 && simOverlayState.simulatedProgress < 0.96) {
      if (simOverlayState.pitStopSecondsRemaining > 0) {
        simOverlayState.status = "IN_PIT_BOX";
        // Stationary vehicle halt for tyre change
        simOverlayState.pitStopSecondsRemaining -= dt;

        if (simOverlayState.pitStopSecondsRemaining <= 0) {
          // Tyre swap complete!
          simOverlayState.currentCompound = simOverlayState.newCompound;
          simOverlayState.tyreAge = 0;
          simOverlayState.pitStopCount++;
          simOverlayState.pitRequested = false;
          simOverlayState.isCurrentlyBoxing = false;
          simOverlayState.status = "EXITING_PITS";
          simOverlayState.simulatedProgress = 0.961;

          // Reset Box Button back to ready state for subsequent pit stops!
          const btnTitle = document.getElementById('btnBoxTitle');
          const btnSub = document.getElementById('btnBoxSubtitle');
          const btnBox = document.getElementById('btnBoxThisLap');
          if (btnTitle) btnTitle.innerText = "🚨 BOX THIS LAP";
          if (btnSub) btnSub.innerText = `FIT NEW ${selectedTargetCompound.toUpperCase()} TYRES • SWITCH TO SIMULATION`;
          if (btnBox) btnBox.classList.remove('box-confirmed');

          // Update Hero Card in right panel
          const heroTyre = document.getElementById('heroTyreBadge');
          const heroAge = document.getElementById('heroTyreAge');
          const heroPits = document.getElementById('heroPitCount');
          const compInfo = getCompoundInfo(simOverlayState.currentCompound);
          if (heroTyre) {
            heroTyre.innerText = compInfo.tag;
            heroTyre.className = `compound-tag ${compInfo.cssClass}`;
          }
          if (heroAge) heroAge.innerText = "L0";
          if (heroPits) heroPits.innerText = `${simOverlayState.pitStopCount} PITS`;

          renderTimingTowerTable();
        }
      } else {
        simOverlayState.status = "EXITING_PITS";
        simOverlayState.simulatedProgress += baseRate * 0.35 * dt;
      }
    }
    // C) Pit Exit Lane (> 0.96 or initial straight < 0.06)
    else if (simOverlayState.simulatedProgress >= 0.96 || simOverlayState.simulatedProgress < 0.06) {
      simOverlayState.status = "EXITING_PITS";
      simOverlayState.simulatedProgress += baseRate * 0.35 * dt;
    }
    // D) Prior to pit entrance on this lap (progress < 0.88)
    else {
      simOverlayState.status = "RACING";
      simOverlayState.simulatedProgress += baseRate * dt;
    }
  } else {
    // Normal Racing or rejoining after pit exit
    if (simOverlayState.status === "EXITING_PITS") {
      if (simOverlayState.simulatedProgress >= 0.06 && simOverlayState.simulatedProgress < 0.88) {
        simOverlayState.status = "RACING";
      } else {
        simOverlayState.simulatedProgress += baseRate * 0.35 * dt;
      }
    } else {
      simOverlayState.status = "RACING";
      simOverlayState.simulatedProgress += baseRate * dt;
    }
  }

  // 4. Lap Rollover (crossing Start/Finish line)
  if (simOverlayState.simulatedProgress >= 1.0) {
    simOverlayState.simulatedProgress %= 1.0;
    simOverlayState.currentLap++;
    simOverlayState.tyreAge++;

    // Update Hero Card tyre age
    const heroAge = document.getElementById('heroTyreAge');
    if (heroAge) heroAge.innerText = `L${simOverlayState.tyreAge}`;
  }

  // 5. Update Simulated Driver in lerpDriversMap with realistic coordinates
  const driverState = lerpDriversMap.get(simOverlayState.driverNumber);
  if (driverState) {
    let coords = getCoordsForProgress(simOverlayState.simulatedProgress);
    if (simOverlayState.status !== "RACING") {
      coords = getPitLaneCoords(simOverlayState.simulatedProgress, coords);
    }

    driverState.currentX = coords.x;
    driverState.currentY = coords.y;
    driverState.targetX = coords.x;
    driverState.targetY = coords.y;
    driverState.progress = simOverlayState.simulatedProgress;
    driverState.current_lap = simOverlayState.currentLap;
    driverState.compound = simOverlayState.currentCompound;
    driverState.tyre_age = simOverlayState.tyreAge;
    driverState.pit_stop_count = simOverlayState.pitStopCount;
    driverState.isPitLap = (simOverlayState.status !== "RACING" || simOverlayState.pitRequested);
    driverState.isSimRow = true;
  }
}

// Reset simulation mode and return driver cleanly to live OpenF1 API stream
function exitSimulationMode() {
  simOverlayState.isActive = false;
  simOverlayState.pitRequested = false;
  simOverlayState.isCurrentlyBoxing = false;
  simOverlayState.status = "RACING";

  // Re-sync followed driver with pure DVR state
  if (dvrState.currentTimestamp !== undefined) {
    applyDVRState(dvrState.currentTimestamp);
  }

  const btnTitle = document.getElementById('btnBoxTitle');
  const btnSub = document.getElementById('btnBoxSubtitle');
  const btnBox = document.getElementById('btnBoxThisLap');
  const btnReset = document.getElementById('btnResetToLive');

  if (btnTitle) btnTitle.innerText = "🚨 BOX THIS LAP";
  if (btnSub) btnSub.innerText = `FIT NEW ${selectedTargetCompound.toUpperCase()} TYRES • SWITCH TO SIMULATION`;
  if (btnBox) btnBox.classList.remove('box-confirmed');
  if (btnReset) btnReset.classList.add('hidden');

  const modeTag = document.getElementById('simModeTag');
  if (modeTag) {
    modeTag.className = "telemetry-mode-tag mode-live";
    modeTag.innerText = "● LIVE OPENF1";
  }

  const banner = document.getElementById('simModeBanner');
  if (banner) banner.classList.add('hidden');

  renderTimingTowerTable();
}

// Race Control Official Status & Live Alert Feedback (GREEN / VSC / SAFETY CAR / RED FLAG)
function setTrackStatus(status) {
  currentTrackStatus = status;

  // 1. Update Header Status Badge
  const badge = document.getElementById('statusBadge');
  if (badge) {
    badge.innerText = `TRACK STATUS: ${status.toUpperCase()}`;
    badge.className = 'status-badge ' + (
      status === 'Green' ? 'status-green' :
      status === 'Yellow' ? 'status-vsc' :
      status === 'VirtualSafetyCar' ? 'status-vsc' :
      status === 'SafetyCar' ? 'status-sc' : 'status-red'
    );
  }

  // 2. Update Race Control Status Badge in Conditions Tab
  const rcBadge = document.getElementById('rcTrackStatusBadge');
  if (rcBadge) {
    rcBadge.innerText = (
      status === 'Green' ? '🟢 GREEN — TRACK CLEAR' :
      status === 'VirtualSafetyCar' ? '🟡 VIRTUAL SAFETY CAR (VSC)' :
      status === 'SafetyCar' ? '🟠 SAFETY CAR DEPLOYED' :
      status === 'RedFlag' ? '🔴 RED FLAG — RACE SUSPENDED' : status.toUpperCase()
    );
    rcBadge.className = 'status-badge ' + (
      status === 'Green' ? 'status-green' :
      status === 'VirtualSafetyCar' || status === 'Yellow' ? 'status-vsc' :
      status === 'SafetyCar' ? 'status-sc' : 'status-red'
    );
  }

  // 3. Update Race Control Alert Banner above visualizer
  const alertBanner = document.getElementById('safetyCarAlertBanner');
  const alertTitle = document.getElementById('scAlertTitle');
  const alertDesc = document.getElementById('scAlertDesc');

  if (alertBanner && alertTitle && alertDesc) {
    if (status === 'SafetyCar') {
      alertBanner.className = 'race-control-alert sc-active';
      alertBanner.classList.remove('hidden');
      alertTitle.innerText = '⚠️ SAFETY CAR DEPLOYED';
      alertDesc.innerText = 'FIELD SPEED RESTRICTED TO DELTA • CHEAP PIT STOP ADVANTAGE ACTIVE (-40% PIT LOSS = ~13.5s)';
    } else if (status === 'VirtualSafetyCar' || status === 'Yellow') {
      alertBanner.className = 'race-control-alert vsc-active';
      alertBanner.classList.remove('hidden');
      alertTitle.innerText = '🟡 VIRTUAL SAFETY CAR (VSC)';
      alertDesc.innerText = 'DRIVERS MUST MAINTAIN POSITIVE DELTA • CHEAP PIT STOP ADVANTAGE ACTIVE (-30% PIT LOSS = ~15.8s)';
    } else if (status === 'RedFlag') {
      alertBanner.className = 'race-control-alert red-active';
      alertBanner.classList.remove('hidden');
      alertTitle.innerText = '🔴 RED FLAG — RACE SUSPENDED';
      alertDesc.innerText = 'RACE SUSPENDED • ALL CARS MUST REDUCE SPEED AND RETURN TO PIT LANE';
    } else {
      alertBanner.className = 'race-control-alert hidden';
    }
  }

  // 4. Update Pit Advantage Label in Strategy Console
  const advBadge = document.getElementById('pitAdvantageBadge');
  if (advBadge) {
    if (status === 'SafetyCar') {
      advBadge.className = 'pit-adv-cheap';
      advBadge.innerText = '★ SC DISCOUNT: 13.5s (-9.0s GAIN!)';
    } else if (status === 'VirtualSafetyCar') {
      advBadge.className = 'pit-adv-cheap';
      advBadge.innerText = '★ VSC DISCOUNT: 15.8s (-6.7s GAIN)';
    } else {
      advBadge.className = 'pit-adv-normal';
      advBadge.innerText = 'PIT LOSS: 22.5s (NORMAL)';
    }
  }
}

// Background analytical simulation to update the 4 Key Insight Cards
async function runSingleDriverSimulation(activateVisuals = false, isBoxNow = false) {
  const driverObj = lerpDriversMap.get(followedDriverNo);
  const curLap = driverObj ? (driverObj.current_lap || 1) : 1;
  const pitLap = isBoxNow ? curLap : Math.min(53, curLap + 4);

  const payload = {
    session_key: parseInt(document.getElementById('boundSessionKey')?.value) || 9000,
    driver_number: followedDriverNo,
    pit_lap: pitLap,
    new_compound: selectedTargetCompound,
    current_lap: curLap,
    track_id: currentSessionLocation || "monza",
  };

  try {
    const resp = await fetch('/api/strategy/simulate-driver', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });

    if (resp.ok) {
      const data = await resp.json();
      renderSingleDriverSimResults(data);
    }
  } catch (e) {
    // Non-fatal, local simulation continues regardless
    console.debug("Backend projection fetch completed/skipped:", e);
  }
}

// Render clean, modern KPI cards in the Strategy Console
function renderSingleDriverSimResults(data) {
  if (!data) return;

  const kpiFinish = document.getElementById('kpiFinishPos');
  if (kpiFinish) kpiFinish.innerText = `P${data.projected_finish_position || 1}`;

  const kpiDelta = document.getElementById('kpiDeltaPos');
  if (kpiDelta) {
    const d = data.position_delta || 0;
    kpiDelta.className = `kpi-delta ${d > 0 ? 'pos-gain' : d < 0 ? 'pos-loss' : 'pos-same'}`;
    kpiDelta.innerText = d > 0 ? `▲ ${d} POS` : d < 0 ? `▼ ${Math.abs(d)} POS` : '― SAME';
  }

  const kpiGap = document.getElementById('kpiGapLeader');
  if (kpiGap) kpiGap.innerText = `+${(data.projected_gap_to_leader || 0).toFixed(1)}s`;

  const kpiTraffic = document.getElementById('kpiTrafficRisk');
  const kpiTrafficDesc = document.getElementById('kpiTrafficDesc');
  if (kpiTraffic && data.traffic_risk) {
    if (data.traffic_risk.startsWith('HIGH')) {
      kpiTraffic.innerText = 'DRS TRAIN';
      kpiTraffic.style.color = 'var(--color-red)';
    } else if (data.traffic_risk.startsWith('MEDIUM')) {
      kpiTraffic.innerText = 'MODERATE';
      kpiTraffic.style.color = 'var(--color-yellow)';
    } else {
      kpiTraffic.innerText = 'CLEAN AIR';
      kpiTraffic.style.color = 'var(--color-green)';
    }
  }
  if (kpiTrafficDesc) {
    kpiTrafficDesc.innerText = `+${(data.pit_exit_gap_ahead_seconds || 3.5).toFixed(1)}s TO CAR AHEAD`;
  }

  const kpiUndercut = document.getElementById('kpiUndercut');
  if (kpiUndercut) {
    kpiUndercut.innerText = `+${(data.undercut_advantage_seconds || 1.4).toFixed(1)}s`;
  }
}

function simToggleGhost() {
  simOverlayState.ghostMode = !simOverlayState.ghostMode;
  const btn = document.getElementById('btnToggleGhost');
  if (btn) btn.innerText = simOverlayState.ghostMode ? '👻 GHOST: ON' : '👻 GHOST: OFF';
}

function simTogglePlay() {
  simOverlayState.isPlaying = !simOverlayState.isPlaying;
  const btn = document.getElementById('simPlayPauseBtn');
  if (btn) {
    btn.innerText = simOverlayState.isPlaying ? '⏸ PAUSE SIM' : '▶ PLAY SIM';
    btn.classList.toggle('active-play', simOverlayState.isPlaying);
  }
}

// ==========================================
// 6. STOCHASTIC MONTE CARLO STRATEGY OPTIMIZER
// ==========================================

async function runSimulation() {
  const driverObj = lerpDriversMap.get(followedDriverNo);
  const curLap = (driverObj && driverObj.current_lap) 
    ? driverObj.current_lap 
    : (parseInt(document.getElementById('currentLapInput')?.value) || 20);
  const curCompound = normalizeCompound((driverObj && driverObj.compound) 
    ? driverObj.compound 
    : (selectedTargetCompound || "Medium"));
  const curTyreAge = (driverObj && driverObj.tyre_age !== undefined) 
    ? driverObj.tyre_age 
    : (simOverlayState.tyreAge || 15);
  const mandatoryPit = (driverObj && (driverObj.pit_stop_count || 0) > 0) || false;

  const allDrivers = Array.from(lerpDriversMap.values()).sort((a, b) => a.position - b.position);
  const egoPos = driverObj ? driverObj.position : 1;
  const competitors = allDrivers
    .filter(d => d.driver_number !== followedDriverNo)
    .slice(0, 8)
    .map(d => ({
      driver_name: d.tla || getDriverTla(d),
      team: d.team || getDriverTeam(d),
      current_lap: d.current_lap || curLap,
      gap_to_ego_seconds: (d.position - egoPos) * 1.8,
      current_tire: normalizeCompound(d.compound || "Medium"),
      tire_age_laps: d.tyre_age || 10,
      projected_pit_lap: Math.min(53, curLap + 5)
    }));

  const payload = {
    track: {
      total_laps: simOverlayState.totalLaps || 53,
      base_pit_stop_loss_seconds: 22.5,
      historical_sc_probability: 0.30
    },
    environment: currentTrackStatus === "SafetyCar" ? "SafetyCar" : currentTrackStatus === "VirtualSafetyCar" ? "VirtualSafetyCar" : "Green",
    ego_car: {
      current_lap: curLap,
      current_tire: curCompound,
      tire_age_laps: curTyreAge,
      mandatory_pit_completed: mandatoryPit,
      front_wing_damage: false,
      time_penalty_seconds: 0.0
    },
    competitors: competitors,
    weather_forecast: null
  };

  try {
    const res = await fetch('/simulate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });

    if (res.ok) {
      const data = await res.json();
      renderStrategies(data);
    }
  } catch (err) {
    console.error("Monte Carlo simulation run failed:", err);
  }
}

function renderStrategies(data) {
  const container = document.getElementById('strategyResults');
  const summaryText = document.getElementById('mcSummaryText');
  if (!container) return;
  container.innerHTML = '';

  if (summaryText && data.summary_message) {
    summaryText.innerHTML = `<strong>OPTIMIZER:</strong> ${data.summary_message} ${data.safety_car_advantage_active ? ' • <span style="color: var(--color-yellow)">★ CHEAP PIT WINDOW ACTIVE</span>' : ''}`;
  }

  const allStrats = [];
  if (data.optimal_strategy) {
    allStrats.push({ ...data.optimal_strategy, isOptimal: true });
  }
  if (Array.isArray(data.alternative_strategies)) {
    data.alternative_strategies.forEach(st => allStrats.push({ ...st, isOptimal: false }));
  }

  allStrats.slice(0, 6).forEach(strat => {
    const card = document.createElement('div');
    card.className = `mc-strategy-card ${strat.isOptimal ? 'optimal' : ''}`;

    const metrics = strat.monte_carlo_metrics || {
      expected_time_seconds: strat.projected_total_time_seconds || 2960.0,
      win_probability_pct: strat.isOptimal ? 86.4 : 45.2,
      p10_time_seconds: (strat.projected_total_time_seconds || 2960.0) - 5.2,
      p90_time_seconds: (strat.projected_total_time_seconds || 2960.0) + 6.8
    };

    const expTime = strat.projected_total_time_formatted || formatLapTime(metrics.expected_time_seconds);
    const winProb = metrics.win_probability_pct ? metrics.win_probability_pct.toFixed(1) : "50.0";
    const deltaStr = strat.isOptimal 
      ? '<span class="mc-delta-badge baseline">★ OPTIMAL BASELINE</span>' 
      : `<span class="mc-delta-badge">+${(strat.delta_to_optimal_seconds || 0).toFixed(2)}s</span>`;

    let stintsHtml = '';
    if (!strat.pit_stops || strat.pit_stops.length === 0) {
      stintsHtml = `<div class="stint-step"><span class="compound-tag comp-medium">MED</span><span class="stint-laps">NO PIT STOP REQUIRED (0-STOP)</span></div>`;
    } else {
      strat.pit_stops.forEach((p) => {
        const c = p.new_compound || p.compound || "Hard";
        const cInfo = getCompoundInfo(c);
        stintsHtml += `
          <div class="stint-step">
            <span class="stint-arrow">➔</span>
            <span class="compound-tag ${cInfo.cssClass}">L${p.pit_lap || p.lap} ${cInfo.tag}</span>
          </div>
        `;
      });
    }

    const firstStopCompound = (strat.pit_stops && strat.pit_stops.length > 0) 
      ? (strat.pit_stops[0].new_compound || strat.pit_stops[0].compound || "Hard")
      : "Hard";
    const firstStopLap = (strat.pit_stops && strat.pit_stops.length > 0)
      ? (strat.pit_stops[0].pit_lap || strat.pit_stops[0].lap || 22)
      : 22;

    card.innerHTML = `
      <div class="mc-card-header">
        <div class="mc-strat-title">
          ${strat.isOptimal ? '<span class="mc-badge-optimal">OPTIMAL</span>' : ''}
          <span class="mc-strat-name">${strat.name}</span>
        </div>
        ${deltaStr}
      </div>

      <div class="mc-stint-flow">
        ${stintsHtml}
      </div>

      <div class="mc-metrics-grid">
        <div class="mc-metric-item">
          <span class="mc-label">EXP. TIME</span>
          <span class="mc-val">${expTime}</span>
        </div>
        <div class="mc-metric-item">
          <span class="mc-label">WIN PROB</span>
          <span class="mc-val" style="color: var(--color-green)">${winProb}%</span>
        </div>
        <div class="mc-metric-item">
          <span class="mc-label">P10 (CEILING)</span>
          <span class="mc-val">${formatLapTime(metrics.p10_time_seconds)}</span>
        </div>
        <div class="mc-metric-item">
          <span class="mc-label">P90 (RISK)</span>
          <span class="mc-val">${formatLapTime(metrics.p90_time_seconds)}</span>
        </div>
      </div>

      <div class="mc-footer">
        <div class="mc-sub-losses">
          <span>PIT LOSS: <strong>${(strat.total_pit_stop_time_loss_seconds || 22.5).toFixed(1)}s</strong></span> • 
          <span>DEG LOSS: <strong>${(strat.total_degradation_loss_seconds || 21.0).toFixed(1)}s</strong></span>
        </div>
        <button type="button" class="btn-apply-strat" onclick="applyMonteCarloStrategy('${firstStopCompound}', ${firstStopLap})">
          SELECT STRATEGY
        </button>
      </div>
    `;

    container.appendChild(card);
  });
}

function applyMonteCarloStrategy(compound, pitLap) {
  selectTargetCompound(compound);
  const pitInput = document.getElementById('userPitLapInput');
  if (pitInput) pitInput.value = pitLap;
  switchTab('tabStrategy');
}
