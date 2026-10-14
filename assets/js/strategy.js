// ==========================================
// 5. STRATEGY ENGINE & SIMULATION CONTROLLER
// ==========================================

// Phase 6: Simulated Trajectory Overlay State
const simOverlayState = {
  isActive: false,
  driverNumber: 1,
  driverTla: "NOR",
  ghostMode: true,
  isPlaying: true,
  playbackSpeed: 1,
  simulatedProgress: 0.0,
  activeTimelineIndex: 0,
  timeline: [],
  totalLaps: 53,
  currentLap: 20,
  pitLap: 22,
  newCompound: "HARD",
  projectedFinishPos: 2,
  posDelta: 1,
  isCurrentlyBoxing: false
};

async function triggerBoxThisLap() {
  const curLap = parseInt(document.getElementById('currentLapInput').value) || 20;
  const pitRange = document.getElementById('userPitLapRange');
  const pitDisp = document.getElementById('userPitLapDisplay');
  const pitInp = document.getElementById('userPitLapInput');
  if (pitRange && pitDisp && pitInp) {
    pitRange.value = curLap;
    pitInp.value = curLap;
    pitDisp.innerText = `L${curLap} (BOX NOW)`;
  }

  simOverlayState.isCurrentlyBoxing = true;
  simOverlayState.pitLap = curLap;

  // Immediately reflect pit entry in timing tower
  const driverObj = lerpDriversMap.get(followedDriverNo);
  if (driverObj) {
    driverObj.isPitLap = true;
    driverObj.pit_stop_count = (driverObj.pit_stop_count || 0) + 1;
    renderTimingTowerTable();
  }

  await runSingleDriverSimulation(true);
}

async function runSingleDriverSimulation(isBoxNow = false) {
  const curLap = parseInt(document.getElementById('currentLapInput').value) || 20;
  const pitLap = isBoxNow ? curLap : (parseInt(document.getElementById('userPitLapInput').value) || (curLap + 4));
  const newCompound = document.getElementById('userNewCompoundSelect').value || "HARD";

  const payload = {
    session_key: parseInt(document.getElementById('boundSessionKey').value) || 9000,
    driver_number: followedDriverNo,
    pit_lap: pitLap,
    new_compound: newCompound,
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
    console.error("Single driver simulation failed", e);
  }
}

function renderSingleDriverSimResults(data) {
  const card = document.getElementById('driverSimResultCard');
  if (!card) return;
  card.classList.remove('hidden');

  const title = document.getElementById('simSummaryTitle');
  if (title) title.innerText = data.is_box_this_lap ? '🚨 BOX THIS LAP EXECUTED' : 'TARGET STRATEGY PROJECTION';

  const deltaEl = document.getElementById('simSummaryDelta');
  if (deltaEl) {
    const d = data.position_delta;
    deltaEl.className = d > 0 ? 'pos-gain' : (d < 0 ? 'pos-loss' : 'pos-same');
    deltaEl.innerText = d > 0 ? `▲${d}` : (d < 0 ? `▼${Math.abs(d)}` : '―');
  }

  const finishPos = document.getElementById('simFinishPos');
  if (finishPos) finishPos.innerText = `P${data.projected_finish_position}`;

  const gapWin = document.getElementById('simGapWinner');
  if (gapWin) gapWin.innerText = `+${data.projected_gap_to_leader}s`;

  const winProb = document.getElementById('simWinProb');
  if (winProb) winProb.innerText = `${data.monte_carlo_metrics.win_probability_pct}%`;

  const undercutAdv = document.getElementById('simUndercutAdv');
  if (undercutAdv) undercutAdv.innerText = `+${data.undercut_advantage_seconds}s`;

  // Traffic Window
  const trafficBadge = document.getElementById('simTrafficRiskBadge');
  const trafficDesc = document.getElementById('simTrafficDesc');
  if (trafficBadge && trafficDesc) {
    trafficDesc.innerText = data.traffic_risk;
    if (data.traffic_risk.startsWith('HIGH')) {
      trafficBadge.className = 'traffic-badge-high';
      trafficBadge.innerText = 'DRS TRAIN RISK';
    } else if (data.traffic_risk.startsWith('MEDIUM')) {
      trafficBadge.className = 'traffic-badge-mid';
      trafficBadge.innerText = 'MODERATE GAP';
    } else {
      trafficBadge.className = 'traffic-badge-low';
      trafficBadge.innerText = 'CLEAN AIR';
    }
  }

  // Emerges Behind & Ahead pills
  const behindContainer = document.getElementById('simEmergesBehindPills');
  if (behindContainer) {
    behindContainer.innerHTML = (data.emerges_behind || []).map(d => `<span class="driver-pill">${d}</span>`).join('') || '<span style="color: var(--text-muted); font-size: 8px;">None (Leading field)</span>';
  }

  const aheadContainer = document.getElementById('simEmergesAheadPills');
  if (aheadContainer) {
    aheadContainer.innerHTML = (data.emerges_ahead || []).map(d => `<span class="driver-pill">${d}</span>`).join('') || '<span style="color: var(--text-muted); font-size: 8px;">None</span>';
  }

  // Monte Carlo Distribution
  const p10 = document.getElementById('simP10');
  const p50 = document.getElementById('simP50');
  const p90 = document.getElementById('simP90');
  const riskScore = document.getElementById('simRiskScore');
  if (p10) p10.innerText = formatLapTime(data.monte_carlo_metrics.p10_time_seconds);
  if (p50) p50.innerText = formatLapTime(data.monte_carlo_metrics.expected_time_seconds);
  if (p90) p90.innerText = formatLapTime(data.monte_carlo_metrics.p90_time_seconds);
  if (riskScore) riskScore.innerText = `${data.monte_carlo_metrics.risk_score}s`;

  // Teammate & Dynamic ODI
  const teammateEl = document.getElementById('simTeammateBenchmark');
  if (teammateEl) {
    teammateEl.innerText = `Pace blended with #${data.teammate_tla} (30% weight) • Base: ${formatLapTime(data.blended_pace_seconds)}/lap`;
  }
  const odiEl = document.getElementById('simDynamicOdiInfo');
  if (odiEl) {
    odiEl.innerText = `Dynamic ODI: ${data.dynamic_odi_rating} • Undercut: +${data.undercut_advantage_seconds}s`;
  }

  // Phase 6: Activate visual simulation trajectory on track canvas and timing tower
  activateSimulationTrajectory(data);
}

function activateSimulationTrajectory(data) {
  if (!data) return;
  simOverlayState.isActive = true;
  simOverlayState.driverNumber = data.driver_number;
  simOverlayState.driverTla = data.driver_tla || "NOR";
  simOverlayState.currentLap = data.current_lap || 20;
  simOverlayState.pitLap = data.pit_lap || 22;
  simOverlayState.newCompound = data.new_compound || "HARD";
  simOverlayState.projectedFinishPos = data.projected_finish_position || 1;
  simOverlayState.posDelta = data.position_delta || 0;
  simOverlayState.timeline = data.lap_timeline || [];
  simOverlayState.activeTimelineIndex = 0;
  simOverlayState.isPlaying = true;
  simOverlayState.isCurrentlyBoxing = (data.is_box_this_lap === true || data.pit_lap === data.current_lap);

  // Initialize simulated dot progress from followed driver's current progress
  const currentDriver = lerpDriversMap.get(data.driver_number);
  if (currentDriver && currentDriver.progress !== undefined && currentDriver.progress !== null) {
    simOverlayState.simulatedProgress = currentDriver.progress;
  } else {
    simOverlayState.simulatedProgress = 0.5;
  }

  // Show Banner and Scrubber
  const banner = document.getElementById('simModeBanner');
  const scrubber = document.getElementById('simScrubberBar');
  if (banner) banner.classList.remove('hidden');
  if (scrubber) scrubber.classList.remove('hidden');

  // Update Banner UI
  const dEl = document.getElementById('simBannerDriver');
  if (dEl) dEl.innerText = `#${data.driver_number} ${data.driver_tla || getDriverTla({ driver_number: data.driver_number })}`;
  const pEl = document.getElementById('simBannerPlan');
  if (pEl) {
    pEl.innerText = simOverlayState.isCurrentlyBoxing 
      ? `(🚨 BOX NOW ➔ ${data.new_compound})` 
      : `(BOX L${data.pit_lap} ➔ ${data.new_compound})`;
  }
  const posEl = document.getElementById('simBannerPos');
  if (posEl) {
    const sign = data.position_delta >= 0 ? '▲' : '▼';
    posEl.className = data.position_delta >= 0 ? 'pos-gain' : 'pos-loss';
    posEl.innerText = `PROJ P${data.projected_finish_position} (${sign}${Math.abs(data.position_delta)})`;
  }

  // Configure Lap Scrubber
  const lapRange = document.getElementById('simLapScrubber');
  if (lapRange) {
    lapRange.min = 0;
    lapRange.max = Math.max(0, simOverlayState.timeline.length - 1);
    lapRange.value = 0;
  }

  updateSimScrubberUI();
  renderTimingTowerTable();
}

function exitSimulationMode() {
  simOverlayState.isActive = false;
  simOverlayState.isCurrentlyBoxing = false;
  const banner = document.getElementById('simModeBanner');
  const scrubber = document.getElementById('simScrubberBar');
  if (banner) banner.classList.add('hidden');
  if (scrubber) scrubber.classList.add('hidden');
  renderTimingTowerTable();
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

function simSetSpeed(speed) {
  simOverlayState.playbackSpeed = speed;
  [1, 2, 5].forEach(s => {
    const btn = document.getElementById(`simSpeed${s}`);
    if (btn) btn.classList.toggle('active', s === speed);
  });
}

function simOnLapScrub(val) {
  const idx = parseInt(val) || 0;
  simOverlayState.activeTimelineIndex = Math.max(0, Math.min(simOverlayState.timeline.length - 1, idx));
  updateSimScrubberUI();
  renderTimingTowerTable();
}

function updateSimScrubberUI() {
  const snap = simOverlayState.timeline[simOverlayState.activeTimelineIndex];
  const curLap = snap ? snap.lap_number : (simOverlayState.currentLap + simOverlayState.activeTimelineIndex);
  const totalLaps = 53;

  const lapDisp = document.getElementById('simLapDisplay');
  if (lapDisp) lapDisp.innerText = `LAP ${curLap} / ${totalLaps}`;

  const bannerLap = document.getElementById('simBannerLap');
  if (bannerLap) bannerLap.innerText = `LAP ${curLap} / ${totalLaps}`;

  const lapRange = document.getElementById('simLapScrubber');
  if (lapRange && parseInt(lapRange.value) !== simOverlayState.activeTimelineIndex) {
    lapRange.value = simOverlayState.activeTimelineIndex;
  }

  if (snap) {
    const posEl = document.getElementById('simBannerPos');
    if (posEl) {
      posEl.innerText = `PROJ P${snap.projected_position}${snap.is_pit_lap ? ' (BOX NOW)' : ''}`;
    }
  }
}

// Multi-driver simulation runner
async function runMultiDriverSimulation() {
  const curLap = parseInt(document.getElementById('currentLapInput').value) || 20;
  const targetPitLap = parseInt(document.getElementById('userPitLapInput').value) || 22;
  const newCompound = document.getElementById('userNewCompoundSelect').value || "HARD";
  const r1Reaction = document.getElementById('rival1Reaction').value || "CoverUndercut";
  const r2Reaction = document.getElementById('rival2Reaction').value || "ExtendStint";

  const allDrivers = Array.from(lerpDriversMap.values()).sort((a, b) => a.position - b.position);
  const topContenders = allDrivers.slice(0, 5);

  const simDrivers = topContenders.map(d => {
    const isFollowed = (d.driver_number === followedDriverNo);
    let plannedLaps = [];
    let targetComps = [];
    let reaction = "Standard";

    if (isFollowed) {
      plannedLaps = [targetPitLap];
      targetComps = [newCompound];
      reaction = "Standard";
    } else if (d.position === 1 || d.position === 2) {
      plannedLaps = [targetPitLap + 2];
      targetComps = ["HARD"];
      reaction = r1Reaction;
    } else {
      plannedLaps = [targetPitLap + 8];
      targetComps = ["MEDIUM"];
      reaction = r2Reaction;
    }

    return {
      driver_id: d.tla,
      team: d.team,
      current_position: d.position,
      gap_to_leader_seconds: (d.position - 1) * 2.2,
      current_compound: d.compound || "MEDIUM",
      tyre_age: d.tyre_age || 15,
      planned_pit_laps: plannedLaps,
      target_compounds: targetComps,
      rival_reaction_mode: reaction
    };
  });

  const payload = {
    session_key: parseInt(document.getElementById('boundSessionKey').value) || 9000,
    total_laps: 53,
    current_lap: curLap,
    track_id: currentSessionLocation || "monza",
    drivers: simDrivers
  };

  try {
    const resp = await fetch('/api/simulate-multi', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });

    if (resp.ok) {
      const data = await resp.json();
      renderMultiDriverSimResults(data);
    }
  } catch (e) {
    console.error("Multi-driver simulation request failed", e);
  }
}

function renderMultiDriverSimResults(data) {
  const container = document.getElementById('multiSimResultsContainer');
  const content = document.getElementById('multiSimResultsContent');
  if (!container || !content) return;

  container.classList.remove('hidden');

  let rows = '';
  (data.driver_outcomes || []).forEach(o => {
    const delta = o.initial_position - o.final_position;
    const deltaClass = delta > 0 ? 'pos-gain' : (delta < 0 ? 'pos-loss' : 'pos-same');
    const deltaSymbol = delta > 0 ? `▲${delta}` : (delta < 0 ? `▼${Math.abs(delta)}` : '―');

    rows += `
      <tr>
        <td style="font-weight: 800;">P${o.final_position}</td>
        <td><strong>${o.driver_id}</strong> <span style="font-size: 8px; color: var(--text-muted);">${o.team}</span></td>
        <td class="${deltaClass}">${deltaSymbol}</td>
        <td>+${o.gap_to_winner.toFixed(1)}s</td>
        <td><span class="pit-tag">${o.pit_stops} PITS</span></td>
      </tr>
    `;
  });

  content.innerHTML = `
    <table class="multi-sim-results-table">
      <thead>
        <tr>
          <th>FIN</th>
          <th>DRIVER</th>
          <th>DELTA</th>
          <th>GAP</th>
          <th>STRAT</th>
        </tr>
      </thead>
      <tbody>
        ${rows}
      </tbody>
    </table>
    <div class="narrative-box">
      <strong>AI RACE ENGINEER SUMMARY:</strong><br>
      ${data.race_narrative}
    </div>
  `;
}

// Global AI strategy optimization call
async function runSimulation() {
  const payload = {
    car: {
      team: "McLaren",
      base_pace: 88.0,
      tire_deg_mult: 1.0,
      pit_crew_avg: 2.4,
      pit_crew_std_dev: 0.2
    },
    race_state: {
      total_laps: 53,
      current_lap: parseInt(document.getElementById('currentLapInput').value) || 20,
      current_compound: document.getElementById('tireSelect').value || "Medium",
      current_tire_age: parseInt(document.getElementById('tireAgeInput').value) || 15,
      current_position: 1,
      gap_to_leader: 0.0,
      track_status: document.getElementById('envSelect').value || "Green",
      weather: {
        current: { air_temp: 24.0, track_temp: 38.0, rain_intensity: 0.0, weather_type: "Dry" },
        forecast: []
      },
      competitors: []
    },
    track_params: {
      base_pit_loss: 24.0,
      tire_wear_mult: 1.0,
      drs_zones: 2,
      fuel_effect_per_lap: 0.035
    }
  };

  try {
    const res = await fetch('/simulate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });

    if (res.ok) {
      const data = await res.json();
      renderStrategies(data.strategies);
    }
  } catch (err) {
    console.error("Simulation run failed:", err);
  }
}

function renderStrategies(strategies) {
  const container = document.getElementById('strategyResults');
  if (!container) return;
  container.innerHTML = '';

  (strategies || []).forEach((strat, idx) => {
    const isOpt = (idx === 0);
    const card = document.createElement('div');
    card.className = `matrix-card ${isOpt ? 'optimal' : ''}`;

    let stopsDesc = strat.pit_stops.length === 0 
      ? 'NO STOP (0-STOP)' 
      : strat.pit_stops.map(p => `LAP ${p.lap} ➔ ${p.compound}`).join(' | ');

    card.innerHTML = `
      <div class="matrix-title">
        <span>${isOpt ? '★ OPTIMAL ' : ''}${strat.name.toUpperCase()}</span>
        <span>${stopsDesc}</span>
      </div>
      <div style="font-size: 10px; color: var(--text-muted);">${strat.description || 'Estimated multi-stint simulation'}</div>
      <div class="stat-grid">
        <div class="stat-box">
          <div class="stat-label">EXP. TIME</div>
          <div class="stat-val">${formatLapTime(strat.metrics.expected_time_seconds)}</div>
        </div>
        <div class="stat-box">
          <div class="stat-label">WIN PROB</div>
          <div class="stat-val" style="color: var(--color-green)">${strat.metrics.win_probability_pct.toFixed(1)}%</div>
        </div>
        <div class="stat-box">
          <div class="stat-label">P10 (CEILING)</div>
          <div class="stat-val">${formatLapTime(strat.metrics.p10_time_seconds)}</div>
        </div>
        <div class="stat-box">
          <div class="stat-label">P90 (RISK)</div>
          <div class="stat-val">${formatLapTime(strat.metrics.p90_time_seconds)}</div>
        </div>
      </div>
    `;
    container.appendChild(card);
  });
}
