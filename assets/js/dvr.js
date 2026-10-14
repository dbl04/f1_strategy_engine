// ==========================================
// 3. IN-MEMORY DVR TICKER ENGINE
// ==========================================
const dvrState = {
  currentTimestamp: 0,   // Elapsed session playback time in seconds
  startTime: 0,
  endTime: 120,          // 120s buffer
  isPlaying: false,      // Default paused on race entry
  playbackSpeed: 1,      // 1x, 2x, 5x, 10x
  telemetryBuffer: [],   // In-memory dataset pre-loaded from backend
  lastFrameTime: performance.now(),
  lastTowerUpdateSec: -1
};

let baseDriversList = [];
const gpsTimelinesByDriver = new Map();

function dvrPlay() {
  dvrState.isPlaying = true;
  if (typeof simOverlayState !== 'undefined') {
    simOverlayState.isPlaying = true;
  }
  updateDVRControlsUI();
}

function dvrPause() {
  dvrState.isPlaying = false;
  if (typeof simOverlayState !== 'undefined') {
    simOverlayState.isPlaying = false;
  }
  updateDVRControlsUI();
}

function dvrTogglePlay() {
  dvrState.isPlaying = !dvrState.isPlaying;
  if (typeof simOverlayState !== 'undefined') {
    simOverlayState.isPlaying = dvrState.isPlaying;
  }
  updateDVRControlsUI();
}

function dvrSetSpeed(multiplier) {
  dvrState.playbackSpeed = multiplier;
  if (typeof simOverlayState !== 'undefined') {
    simOverlayState.playbackSpeed = multiplier;
  }
  [1, 2, 5, 10].forEach(spd => {
    const btn = document.getElementById(`dvrSpeed${spd}`);
    if (btn) btn.classList.toggle('active', spd === multiplier);
  });
}

function dvrOnScrub(val) {
  const time = parseFloat(val);
  dvrState.currentTimestamp = time;
  applyDVRState(time);
}

function updateDVRControlsUI() {
  const playBtn = document.getElementById('dvrPlayPauseBtn');
  if (playBtn) {
    playBtn.innerText = dvrState.isPlaying ? '⏸ PAUSE' : '▶ PLAY';
    playBtn.classList.toggle('active-play', dvrState.isPlaying);
  }
}

function showLoadingScreen(initialText = "Connecting to OpenF1 Session Pipeline...") {
  const overlay = document.getElementById('sessionLoadingOverlay');
  const statusText = document.getElementById('loadingStatusText');
  const progressBar = document.getElementById('loadingProgressBar');
  const progressPercent = document.getElementById('loadingProgressPercent');
  if (overlay) {
    overlay.classList.remove('hidden');
    if (statusText) statusText.innerText = initialText;
    if (progressBar) progressBar.style.width = '10%';
    if (progressPercent) progressPercent.innerText = '10%';
  }
}

function updateLoadingProgress(percent, text) {
  const statusText = document.getElementById('loadingStatusText');
  const progressBar = document.getElementById('loadingProgressBar');
  const progressPercent = document.getElementById('loadingProgressPercent');
  if (statusText && text) statusText.innerText = text;
  if (progressBar) progressBar.style.width = `${percent}%`;
  if (progressPercent) progressPercent.innerText = `${percent}%`;
}

function hideLoadingScreen() {
  const overlay = document.getElementById('sessionLoadingOverlay');
  if (overlay) {
    updateLoadingProgress(100, "Calibration Complete. Launching Digital Twin...");
    setTimeout(() => {
      overlay.classList.add('hidden');
    }, 400);
  }
}

// Build Continuous In-Memory High-Precision DVR Buffer
function buildSessionDVRBuffer(initialDrivers, trackStatus, trackTemp, airTemp, rawLaps, rawStints) {
  const defaultDrivers = [
    { driver_number: 1, tla: "NOR", team: "McLaren", initialPos: 1, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.998 },
    { driver_number: 3, tla: "VER", team: "Red Bull Racing", initialPos: 2, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.995 },
    { driver_number: 81, tla: "PIA", team: "McLaren", initialPos: 3, compound: "HARD", initialTyreAge: 0, initialProgress: 0.992 },
    { driver_number: 16, tla: "LEC", team: "Ferrari", initialPos: 4, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.989 },
    { driver_number: 44, tla: "HAM", team: "Ferrari", initialPos: 5, compound: "HARD", initialTyreAge: 0, initialProgress: 0.986 },
    { driver_number: 63, tla: "RUS", team: "Mercedes", initialPos: 6, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.983 },
    { driver_number: 12, tla: "ANT", team: "Mercedes", initialPos: 7, compound: "SOFT", initialTyreAge: 0, initialProgress: 0.980 },
    { driver_number: 14, tla: "ALO", team: "Aston Martin", initialPos: 8, compound: "HARD", initialTyreAge: 0, initialProgress: 0.977 },
    { driver_number: 18, tla: "STR", team: "Aston Martin", initialPos: 9, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.974 },
    { driver_number: 6, tla: "HAD", team: "Red Bull Racing", initialPos: 10, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.971 },
    { driver_number: 10, tla: "GAS", team: "Alpine", initialPos: 11, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.968 },
    { driver_number: 43, tla: "COL", team: "Alpine", initialPos: 12, compound: "HARD", initialTyreAge: 0, initialProgress: 0.965 },
    { driver_number: 23, tla: "ALB", team: "Williams", initialPos: 13, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.962 },
    { driver_number: 55, tla: "SAI", team: "Williams", initialPos: 14, compound: "HARD", initialTyreAge: 0, initialProgress: 0.959 },
    { driver_number: 30, tla: "LAW", team: "Racing Bulls", initialPos: 15, compound: "HARD", initialTyreAge: 0, initialProgress: 0.956 },
    { driver_number: 41, tla: "LIN", team: "Racing Bulls", initialPos: 16, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.953 },
    { driver_number: 31, tla: "OCO", team: "Haas F1 Team", initialPos: 17, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.950 },
    { driver_number: 87, tla: "BEA", team: "Haas F1 Team", initialPos: 18, compound: "SOFT", initialTyreAge: 0, initialProgress: 0.947 },
    { driver_number: 5, tla: "BOR", team: "Audi", initialPos: 19, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.944 },
    { driver_number: 27, tla: "HUL", team: "Audi", initialPos: 20, compound: "HARD", initialTyreAge: 0, initialProgress: 0.941 },
    { driver_number: 11, tla: "PER", team: "Cadillac", initialPos: 21, compound: "MEDIUM", initialTyreAge: 0, initialProgress: 0.938 },
    { driver_number: 77, tla: "BOT", team: "Cadillac", initialPos: 22, compound: "HARD", initialTyreAge: 0, initialProgress: 0.935 },
  ];

  const driverPool = (initialDrivers && initialDrivers.length > 0) ? initialDrivers : defaultDrivers;
  let maxSessionLaps = 53;
  let maxSessionDuration = 0;

  // Group real laps per driver
  const lapsByDriver = new Map();
  if (Array.isArray(rawLaps) && rawLaps.length > 0) {
    rawLaps.forEach(lap => {
      const dNo = Number(lap.driver_number);
      if (!isNaN(dNo)) {
        if (!lapsByDriver.has(dNo)) lapsByDriver.set(dNo, []);
        lapsByDriver.get(dNo).push(lap);
      }
    });
  }

  // Group real stints per driver
  const stintsByDriver = new Map();
  if (Array.isArray(rawStints) && rawStints.length > 0) {
    rawStints.forEach(stint => {
      const dNo = Number(stint.driver_number);
      if (!isNaN(dNo)) {
        if (!stintsByDriver.has(dNo)) stintsByDriver.set(dNo, []);
        stintsByDriver.get(dNo).push({
          stint_number: stint.stint_number || 1,
          compound: stint.compound ? stint.compound.toUpperCase() : "MEDIUM",
          tyre_age_at_start: (stint.tyre_age_at_start !== undefined && stint.tyre_age_at_start !== null) ? stint.tyre_age_at_start : 0,
          lap_start: (stint.lap_start !== undefined && stint.lap_start !== null && stint.lap_start > 0) ? stint.lap_start : 1,
          lap_end: (stint.lap_end !== undefined && stint.lap_end !== null && stint.lap_end > 0) ? stint.lap_end : 999
        });
      }
    });
  }

  baseDriversList = driverPool.map((d, idx) => {
    const dNo = Number(d.driver_number);
    const gridSlotProgress = Math.max(0.90, 0.998 - (idx * 0.003));
    const dLaps = lapsByDriver.get(dNo) || [];
    dLaps.sort((a, b) => a.lap_number - b.lap_number);

    const dStints = stintsByDriver.get(dNo) || [];
    dStints.sort((a, b) => (a.stint_number || a.lap_start || 0) - (b.stint_number || b.lap_start || 0));

    let initialCompound = d.compound || "MEDIUM";
    let initialTyreAge = d.tyre_age || 0;
    if (dStints.length > 0) {
      initialCompound = dStints[0].compound || initialCompound;
      initialTyreAge = (dStints[0].tyre_age_at_start !== undefined && dStints[0].tyre_age_at_start !== null) ? dStints[0].tyre_age_at_start : initialTyreAge;
    }

    let cumTime = 0;
    const timelineLaps = [];
    dLaps.forEach(l => {
      const totalDur = (l.lap_duration && l.lap_duration > 30) ? l.lap_duration : (88.0 + (idx * 0.35));
      let s1 = l.duration_sector_1;
      let s2 = l.duration_sector_2;
      let s3 = l.duration_sector_3;

      const standardPace = 88.0 + (idx * 0.35);
      if (!s1 || !s2 || !s3 || (s1 <= 0) || (s2 <= 0) || (s3 <= 0)) {
        s1 = totalDur * 0.33;
        s2 = totalDur * 0.35;
        s3 = totalDur - (s1 + s2);
      }

      timelineLaps.push({
        lap_number: l.lap_number,
        lap_duration: totalDur,
        s1_time: s1,
        s2_time: s2,
        s3_time: s3,
        startTime: cumTime,
        endTime: cumTime + totalDur,
        is_pit_out_lap: l.is_pit_out_lap || false
      });
      cumTime += totalDur;
    });

    if (cumTime > maxSessionDuration) {
      maxSessionDuration = cumTime;
    }
    if (timelineLaps.length > maxSessionLaps) {
      maxSessionLaps = timelineLaps.length;
    }

    const resolvedTla = getDriverTla(d);
    const resolvedTeam = getDriverTeam(d);

    return {
      driver_number: dNo,
      tla: resolvedTla,
      team: resolvedTeam,
      gridSlotProgress: (d.initialProgress !== undefined) ? d.initialProgress : gridSlotProgress,
      timelineLaps: timelineLaps,
      stints: dStints,
      initialCompound: initialCompound,
      initialTyreAge: initialTyreAge,
      pit_stop_count: d.pit_stop_count || 0
    };
  });

  dvrState.startTime = 0;
  dvrState.endTime = Math.max(120, maxSessionDuration);
  dvrState.currentTimestamp = 0;

  const scrubber = document.getElementById('dvrScrubber');
  if (scrubber) {
    scrubber.min = 0;
    scrubber.max = dvrState.endTime.toFixed(1);
    scrubber.value = 0;
  }

  // Pre-seed interpolation map
  baseDriversList.forEach((d, idx) => {
    const coords = getCoordsForProgress(d.gridSlotProgress);
    lerpDriversMap.set(d.driver_number, {
      driver_number: d.driver_number,
      tla: d.tla || getDriverTla(d),
      team: d.team || getDriverTeam(d),
      position: idx + 1,
      gap: idx === 0 ? "LEADER" : `+${(idx * 1.5).toFixed(1)}s`,
      interval: idx === 0 ? "―" : `+1.5s`,
      compound: d.initialCompound,
      tyre_age: d.initialTyreAge,
      pit_stop_count: 0,
      drs_active: false,
      last_lap_time: 88.4,
      currentX: coords.x,
      currentY: coords.y,
      targetX: coords.x,
      targetY: coords.y,
      progress: d.gridSlotProgress
    });
  });

  applyDVRState(0);
}

function applyDVRState(timestamp) {
  let leaderLapNumber = 1;
  const activeLapsByDriver = new Map();

  baseDriversList.forEach(driver => {
    const dNo = driver.driver_number;
    const gpsTimeline = gpsTimelinesByDriver.get(dNo);

    if (gpsTimeline && gpsTimeline.length > 0) {
      let low = 0;
      let high = gpsTimeline.length - 1;
      while (low <= high) {
        const mid = (low + high) >> 1;
        if (gpsTimeline[mid].t <= timestamp) low = mid + 1;
        else high = mid - 1;
      }
      const idx1 = Math.max(0, Math.min(gpsTimeline.length - 1, low - 1));
      const idx2 = Math.min(gpsTimeline.length - 1, idx1 + 1);
      const p1 = gpsTimeline[idx1];
      const p2 = gpsTimeline[idx2];
      const span = p2.t - p1.t;
      const frac = (span > 0.001) ? (timestamp - p1.t) / span : 0;

      const normX = p1.x + (p2.x - p1.x) * frac;
      const normY = p1.y + (p2.y - p1.y) * frac;
      const s = getLapProgressForCoords(normX, normY);

      activeLapsByDriver.set(dNo, {
        lap_number: Math.floor(timestamp / 88) + 1,
        progress: s,
        targetX: normX,
        targetY: normY,
        compound: driver.initialCompound,
        tyre_age: Math.floor(timestamp / 88),
        pit_stop_count: 0,
        last_lap_time: 88.2
      });
      return;
    }

    // Mathematical timeline lap interpolation
    const laps = driver.timelineLaps;
    let lap = null;
    for (let i = 0; i < laps.length; i++) {
      if (timestamp >= laps[i].startTime && timestamp <= laps[i].endTime) {
        lap = laps[i];
        break;
      }
    }

    if (!lap && laps.length > 0) {
      if (timestamp < laps[0].startTime) lap = laps[0];
      else lap = laps[laps.length - 1];
    }

    let lapProgress = driver.gridSlotProgress;
    let lapNo = 1;
    let lastLapDur = 88.5;

    if (lap) {
      lapNo = lap.lap_number;
      lastLapDur = lap.lap_duration;
      const tInLap = Math.max(0, Math.min(lap.lap_duration, timestamp - lap.startTime));

      if (tInLap < lap.s1_time) {
        lapProgress = (tInLap / lap.s1_time) * 0.333;
      } else if (tInLap < lap.s1_time + lap.s2_time) {
        lapProgress = 0.333 + ((tInLap - lap.s1_time) / lap.s2_time) * 0.333;
      } else {
        const s3Time = Math.max(0.1, lap.lap_duration - (lap.s1_time + lap.s2_time));
        lapProgress = 0.666 + ((tInLap - (lap.s1_time + lap.s2_time)) / s3Time) * 0.334;
      }
    } else {
      const pace = 88.0 + (driver.driver_number % 5) * 0.3;
      lapNo = Math.floor(timestamp / pace) + 1;
      lapProgress = (timestamp % pace) / pace;
    }

    // Active stint and tyre calculation
    let currentCompound = driver.initialCompound;
    let currentTyreAge = driver.initialTyreAge + (lapNo - 1);
    let pitCount = 0;

    if (driver.stints && driver.stints.length > 0) {
      for (let sIdx = 0; sIdx < driver.stints.length; sIdx++) {
        const stint = driver.stints[sIdx];
        if (lapNo >= stint.lap_start && lapNo <= stint.lap_end) {
          currentCompound = stint.compound;
          currentTyreAge = stint.tyre_age_at_start + (lapNo - stint.lap_start);
          pitCount = sIdx;
          break;
        }
      }
    }

    const { x, y } = getCoordsForProgress(lapProgress);
    activeLapsByDriver.set(dNo, {
      lap_number: lapNo,
      progress: lapProgress,
      targetX: x,
      targetY: y,
      compound: currentCompound,
      tyre_age: currentTyreAge,
      pit_stop_count: pitCount,
      last_lap_time: lastLapDur
    });
  });

  // Inject Simulated Driver trajectory state into active field before ranking
  if (simOverlayState.isActive) {
    const coords = getCoordsForProgress(simOverlayState.simulatedProgress);
    activeLapsByDriver.set(simOverlayState.driverNumber, {
      lap_number: simOverlayState.currentLap,
      progress: simOverlayState.simulatedProgress,
      targetX: coords.x,
      targetY: coords.y,
      compound: simOverlayState.currentCompound,
      tyre_age: simOverlayState.tyreAge,
      pit_stop_count: simOverlayState.pitStopCount,
      last_lap_time: simOverlayState.lastLapTime || 88.2,
      isSim: true
    });
  }

  // Sort and rank all 22 drivers by track distance: (lap * 1.0) + progress
  const sorted = Array.from(activeLapsByDriver.entries()).map(([dNo, data]) => {
    const totalScore = (data.lap_number * 1.0) + data.progress;
    return { dNo, totalScore, ...data };
  });

  sorted.sort((a, b) => b.totalScore - a.totalScore);

  let leaderScore = sorted.length > 0 ? sorted[0].totalScore : 1.0;
  leaderLapNumber = sorted.length > 0 ? sorted[0].lap_number : 1;

  sorted.forEach((item, rank) => {
    const driverState = lerpDriversMap.get(item.dNo);
    if (!driverState) return;

    const gapSec = (leaderScore - item.totalScore) * 88.0;
    const prevItem = rank > 0 ? sorted[rank - 1] : null;
    const intSec = prevItem ? (prevItem.totalScore - item.totalScore) * 88.0 : 0.0;
    const isOvertakeEligible = (intSec > 0.05 && intSec <= 1.0);

    driverState.position = rank + 1;
    driverState.gap = rank === 0 ? "LEADER" : `+${gapSec.toFixed(1)}s`;
    driverState.interval = rank === 0 ? "―" : `+${intSec.toFixed(1)}s`;
    driverState.drs_active = isOvertakeEligible; // 2026 Overtake mode

    if (simOverlayState.isActive && item.dNo === simOverlayState.driverNumber) {
      driverState.compound = simOverlayState.currentCompound;
      driverState.tyre_age = simOverlayState.tyreAge;
      driverState.pit_stop_count = simOverlayState.pitStopCount;
      driverState.current_lap = simOverlayState.currentLap;
      driverState.progress = simOverlayState.simulatedProgress;
      driverState.isSimRow = true;
      driverState.isPitLap = (simOverlayState.status !== "RACING" || simOverlayState.pitRequested);
      // Simulated driver coordinates are handled directly by the simulation physics engine
    } else {
      driverState.compound = item.compound;
      driverState.tyre_age = item.tyre_age;
      driverState.pit_stop_count = item.pit_stop_count;
      driverState.current_lap = item.lap_number;
      driverState.progress = item.progress;
      driverState.targetX = item.targetX;
      driverState.targetY = item.targetY;
      driverState.last_lap_time = item.last_lap_time;
      driverState.isSimRow = false;
      driverState.isPitLap = false;
    }
  });

  // Sync Header Bar
  const lapCounter = document.getElementById('headerLapCounter');
  if (lapCounter) {
    lapCounter.innerText = `${leaderLapNumber} / ${simOverlayState.totalLaps || 53}`;
  }

  const scrubber = document.getElementById('dvrScrubber');
  if (scrubber && Math.abs(parseFloat(scrubber.value) - timestamp) > 0.4) {
    scrubber.value = timestamp.toFixed(1);
  }
  const timeDisplay = document.getElementById('dvrTimeDisplay');
  if (timeDisplay) {
    timeDisplay.innerText = formatTime(timestamp);
  }

  // Update Timing Tower synchronized with track positions at 2Hz
  const currentSec = Math.floor(timestamp * 2);
  if (currentSec !== dvrState.lastTowerUpdateSec) {
    dvrState.lastTowerUpdateSec = currentSec;
    renderTimingTowerTable();
  }
}
