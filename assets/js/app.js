// ==========================================
// 6. MAIN APPLICATION ORCHESTRATOR & 60 FPS LOOP
// ==========================================

let currentSessionKey = 9000;
let currentSessionLocation = "Monza";
let followedDriverNo = 1;
let activeVisualizer = "2d";
let networkPollInterval = null;

// Driver LERP Interpolation State Map
const lerpDriversMap = new Map();
let refTrackPoints = [];

const radarEngine = new OpenF1RadarEngine('trackCanvas');
const circularRingEngine = new CircularRadarEngine('ringCanvas');

function renderLoop(now) {
  const deltaTime = (now - dvrState.lastFrameTime) / 1000;
  dvrState.lastFrameTime = now;

  // Advance In-Memory DVR Playback Ticker
  if (dvrState.isPlaying && (dvrState.telemetryBuffer.length > 0 || baseDriversList.length > 0)) {
    dvrState.currentTimestamp += deltaTime * dvrState.playbackSpeed;
    if (dvrState.currentTimestamp > dvrState.endTime) {
      dvrState.currentTimestamp = dvrState.startTime; // Loop playback
    }
    applyDVRState(dvrState.currentTimestamp);
  }

  // Advance Simulated Strategy Trajectory Dot and Physical Vehicle Dynamics
  if (simOverlayState.isActive && simOverlayState.isPlaying && dvrState.isPlaying) {
    tickSimulatedDriver(deltaTime);
  }

  // Smooth Linear Interpolation towards targets with teleport guard
  lerpDriversMap.forEach(driver => {
    const distSq = (driver.targetX - driver.currentX) ** 2 + (driver.targetY - driver.currentY) ** 2;
    if (distSq > 0.005) {
      driver.currentX = driver.targetX;
      driver.currentY = driver.targetY;
    } else {
      driver.currentX += (driver.targetX - driver.currentX) * 0.25;
      driver.currentY += (driver.targetY - driver.currentY) * 0.25;
    }
  });

  if (activeVisualizer === '2d') {
    radarEngine.render();
  } else if (activeVisualizer === 'ring') {
    circularRingEngine.render();
  }

  requestAnimationFrame(renderLoop);
}

function switchVisualizer(type) {
  activeVisualizer = type;
  const btn2D = document.getElementById('btnViz2D');
  const btnRing = document.getElementById('btnVizRing');
  if (btn2D) btn2D.classList.toggle('active', type === '2d');
  if (btnRing) btnRing.classList.toggle('active', type === 'ring');

  const c2d = document.getElementById('trackCanvas');
  const cRing = document.getElementById('ringCanvas');

  if (type === '2d') {
    if (c2d) c2d.classList.remove('hidden');
    if (cRing) cRing.classList.add('hidden');
    radarEngine.render();
  } else {
    if (c2d) c2d.classList.add('hidden');
    if (cRing) cRing.classList.remove('hidden');
    circularRingEngine.render();
  }
}

// Navigation & Views
function showLandingView() {
  dvrPause();
  if (networkPollInterval) {
    clearInterval(networkPollInterval);
    networkPollInterval = null;
  }
  document.getElementById('landingView').classList.remove('hidden');
  document.getElementById('dashboardView').classList.add('hidden');
}

async function showDashboardView(sessionKey, location, sessionName) {
  currentSessionKey = sessionKey;
  currentSessionLocation = location;
  document.getElementById('boundSessionKey').value = sessionKey;
  document.getElementById('dashSessionKeyDisplay').innerText = sessionKey;
  document.getElementById('dashSessionTitle').innerText = `${location.toUpperCase()} - ${sessionName.toUpperCase()}`;

  document.getElementById('landingView').classList.add('hidden');
  document.getElementById('dashboardView').classList.remove('hidden');

  showLoadingScreen(`Acquiring High-Frequency GPS Telemetry for ${location} (Session ${sessionKey})...`);

  // Load track geometry and start DVR
  await radarEngine.loadReferenceLapMap(sessionKey);
  await initSessionDVR(sessionKey);

  // Auto-select followed driver or race leader to populate Hero and strategy controls
  const initialDriver = lerpDriversMap.get(followedDriverNo) || lerpDriversMap.get(1) || Array.from(lerpDriversMap.values())[0];
  if (initialDriver) {
    selectFollowedDriver(
      initialDriver.driver_number,
      initialDriver.tla,
      initialDriver.compound,
      initialDriver.tyre_age,
      initialDriver.pit_stop_count,
      initialDriver.team
    );
  } else {
    runSimulation();
    runSingleDriverSimulation(false, false);
  }

  // Start periodic live state polling from OpenF1 (Shielded proxy)
  if (networkPollInterval) clearInterval(networkPollInterval);
  networkPollInterval = setInterval(pollLiveState, 4000);
}

async function pollLiveState() {
  try {
    const res = await fetch(`/api/live-state/${currentSessionKey}`);
    if (res.ok) {
      const state = await res.json();
      
      if (state.track_status) {
        setTrackStatus(state.track_status);
      }

      const odiEl = document.getElementById('headerDynamicOdi');
      if (odiEl && state.current_odi !== undefined) {
        odiEl.innerText = `${state.current_odi.toFixed(2)} (${state.odi_rating || 'MODERATE'})`;
      }
    }
  } catch (err) {
    console.debug("Live state poll skipped/dormant:", err);
  }
}

async function initSessionDVR(sessionKey) {
  try {
    updateLoadingProgress(20, "Fetching OpenF1 Drivers & Grid Lineup...");
    const driversRes = await fetch(`/api/openf1/drivers?session_key=${sessionKey}`);
    const drivers = driversRes.ok ? await driversRes.json() : [];

    updateLoadingProgress(40, "Retrieving Real Session Laps & Timing Records...");
    const lapsRes = await fetch(`/api/openf1/session-laps?session_key=${sessionKey}`);
    const rawLaps = lapsRes.ok ? await lapsRes.json() : [];

    updateLoadingProgress(60, "Acquiring Stint Compounds & Degradation Metadata...");
    const stintsRes = await fetch(`/api/openf1/stints?session_key=${sessionKey}`);
    const rawStints = stintsRes.ok ? await stintsRes.json() : [];

    updateLoadingProgress(75, "Acquiring Official OpenF1 Race Control Incidents...");
    let initialTrackStatus = "Green";
    try {
      const rcRes = await fetch(`/api/openf1/race_control?session_key=${sessionKey}`);
      if (rcRes.ok) {
        const rawRC = await rcRes.json();
        renderRaceControlEvents(rawRC);
        if (rawRC && rawRC.length > 0) {
          const lastActive = rawRC.slice().reverse().find(rc => rc.flag || rc.category === "SafetyCar" || rc.message?.toUpperCase().includes("SAFETY CAR"));
          if (lastActive) {
            const f = (lastActive.flag || lastActive.message || "").toUpperCase();
            if (f.includes("VIRTUAL") || f.includes("VSC")) initialTrackStatus = "VirtualSafetyCar";
            else if (f.includes("SAFETY CAR")) initialTrackStatus = "SafetyCar";
            else if (f.includes("RED")) initialTrackStatus = "RedFlag";
            else if (f.includes("YELLOW")) initialTrackStatus = "VirtualSafetyCar";
          }
        }
      }
    } catch (e) {
      console.debug("Race control fetch skipped:", e);
    }

    updateLoadingProgress(88, "Synchronizing Ambient & Track Weather Feeds...");
    const weatherRes = await fetch(`/api/openf1/weather?session_key=${sessionKey}`);
    let trackTemp = 38.4, airTemp = 24.1;
    if (weatherRes.ok) {
      const wList = await weatherRes.json();
      if (wList && wList.length > 0) {
        trackTemp = wList[wList.length - 1].track_temperature || 38.4;
        airTemp = wList[wList.length - 1].air_temperature || 24.1;
      }
    }

    const trackTempEl = document.getElementById('headerTrackTemp');
    if (trackTempEl) trackTempEl.innerText = `${trackTemp.toFixed(1)}°C`;
    const airTempEl = document.getElementById('headerAirTemp');
    if (airTempEl) airTempEl.innerText = `${airTemp.toFixed(1)}°C`;

    buildSessionDVRBuffer(drivers, initialTrackStatus, trackTemp, airTemp, rawLaps, rawStints);
    setTrackStatus(initialTrackStatus);
  } catch (e) {
    console.error("Failed to initialize session DVR buffer", e);
    buildSessionDVRBuffer([], "GREEN", 38.4, 24.1, [], []);
  }
  hideLoadingScreen();
  dvrPlay();
  updateDVRControlsUI();
}

function renderRaceControlEvents(events) {
  const container = document.getElementById('rcEventsLog');
  if (!container) return;
  if (!events || events.length === 0) {
    container.innerHTML = `
      <div class="rc-log-item">
        <span class="rc-log-flag">🟢</span>
        <span class="rc-log-msg">TRACK CLEAR — ALL SECTORS GREEN</span>
      </div>
    `;
    return;
  }

  let html = '';
  events.slice().reverse().forEach(ev => {
    let flagIcon = '⚪';
    const flag = (ev.flag || ev.category || "").toUpperCase();
    const msg = (ev.message || "").toUpperCase();
    if (flag.includes("GREEN") || msg.includes("CLEAR")) flagIcon = '🟢';
    else if (flag.includes("YELLOW") || msg.includes("YELLOW")) flagIcon = '🟡';
    else if (flag.includes("RED") || msg.includes("RED")) flagIcon = '🔴';
    else if (flag.includes("SAFETY") || msg.includes("SAFETY")) flagIcon = '🟠';
    else if (flag.includes("BLUE")) flagIcon = '🔵';
    else if (flag.includes("CHEQUERED")) flagIcon = '🏁';

    const timeStr = ev.date ? ev.date.substring(11, 19) : 'OFFICIAL';
    html += `
      <div class="rc-log-item">
        <span class="rc-log-time">${timeStr}</span>
        <span class="rc-log-flag">${flagIcon}</span>
        <span class="rc-log-msg">${ev.message || ev.flag || ev.category}</span>
      </div>
    `;
  });
  container.innerHTML = html;
}

function switchTab(tabId) {
  document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
  document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));

  const clickedBtn = Array.from(document.querySelectorAll('.tab-btn')).find(b => b.getAttribute('onclick')?.includes(tabId));
  if (clickedBtn) clickedBtn.classList.add('active');

  const content = document.getElementById(tabId);
  if (content) content.classList.add('active');
}

async function fetchSessions() {
  const container = document.getElementById('accordionContainer');
  if (!container) return;
  container.innerHTML = '<div style="color: var(--text-muted); font-size: 11px;">Loading sessions from OpenF1 pipeline...</div>';

  try {
    const res = await fetch('/api/openf1/sessions');
    if (res.ok) {
      const sessions = await res.json();
      renderAccordions(sessions);
    } else {
      container.innerHTML = '<div style="color: var(--color-red);">Failed to load sessions.</div>';
    }
  } catch (err) {
    container.innerHTML = '<div style="color: var(--color-red);">Network error loading sessions.</div>';
  }
}

function renderAccordions(sessions) {
  const container = document.getElementById('accordionContainer');
  if (!container) return;
  container.innerHTML = '';

  const groups = new Map();
  sessions.forEach(s => {
    const key = `${s.year || 2024} - ${s.country_name || s.location || 'Unknown'}`;
    if (!groups.has(key)) {
      groups.set(key, { location: s.location || 'Circuit', country: s.country_name || 'Grand Prix', sessions: [] });
    }
    groups.get(key).sessions.push(s);
  });

  groups.forEach((group, gpName) => {
    const card = document.createElement('div');
    card.className = 'gp-card';

    let sessionCards = '';
    group.sessions.forEach(sess => {
      const isLive = sess.is_live;
      sessionCards += `
        <div class="sub-session-card ${isLive ? 'is-live' : ''}" onclick="showDashboardView(${sess.session_key}, '${sess.location || 'Monza'}', '${sess.session_name}')">
          <div class="sub-session-type">
            <span>${sess.session_name}</span>
            ${isLive ? '<span style="color: var(--color-red); font-weight: 800;">LIVE</span>' : ''}
          </div>
          <div class="sub-session-date">KEY #${sess.session_key} • ${sess.date_start ? sess.date_start.substring(0, 10) : '2024'}</div>
        </div>
      `;
    });

    card.innerHTML = `
      <div class="gp-card-header" onclick="this.nextElementSibling.classList.toggle('hidden')">
        <div class="gp-title-group">
          <span>🏁</span>
          <span>${gpName.toUpperCase()}</span>
        </div>
        <span style="font-size: 10px; color: var(--color-purple);">(${group.sessions.length} SESSIONS) ▾</span>
      </div>
      <div class="gp-sessions-list">
        ${sessionCards}
      </div>
    `;

    container.appendChild(card);
  });
}

async function replayOpenF1State() {
  try {
    const resp = await fetch('/api/openf1/replay-state', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ session_key: currentSessionKey, speed_multiplier: 1.0 })
    });
    if (resp.ok) {
      alert("Replaying OpenF1 Session telemetry into engine physics buffer.");
    }
  } catch (e) {
    console.error(e);
  }
}

// Global initialization
window.addEventListener('DOMContentLoaded', () => {
  fetchSessions();
  requestAnimationFrame(renderLoop);
});
