// ==========================================
// 4. ENRICHED TIMING TOWER COMPONENT
// ==========================================
function renderTimingTowerTable() {
  const tbody = document.getElementById('timingTowerBody');
  if (!tbody) return;
  tbody.innerHTML = '';

  let drivers = Array.from(lerpDriversMap.values()).map(d => ({ ...d }));

  // If Simulation Overlay is active, inject projected state for the simulated driver
  if (simOverlayState.isActive && simOverlayState.timeline.length > 0) {
    const snap = simOverlayState.timeline[simOverlayState.activeTimelineIndex];
    if (snap) {
      const simIdx = drivers.findIndex(d => d.driver_number === simOverlayState.driverNumber);
      if (simIdx !== -1) {
        drivers[simIdx].position = snap.projected_position;
        drivers[simIdx].gap = snap.projected_position === 1 ? 'LEADER' : `+${snap.gap_to_leader_seconds.toFixed(1)}s`;
        drivers[simIdx].interval = snap.interval_seconds > 0 ? `+${snap.interval_seconds.toFixed(1)}s` : '―';
        drivers[simIdx].compound = snap.compound;
        drivers[simIdx].tyre_age = snap.tyre_age;
        drivers[simIdx].isSimRow = true;
        drivers[simIdx].isPitLap = snap.is_pit_lap || simOverlayState.isCurrentlyBoxing;
        drivers[simIdx].last_lap_time = snap.lap_time_seconds;
        if (snap.is_pit_lap || simOverlayState.isCurrentlyBoxing) {
          drivers[simIdx].pit_stop_count = (drivers[simIdx].pit_stop_count || 0) + 1;
        }
      }
    }
  }

  drivers.sort((a, b) => a.position - b.position);

  drivers.forEach(d => {
    const tr = document.createElement('tr');
    if (d.driver_number === followedDriverNo) tr.className = 'followed-row';
    if (d.isSimRow) tr.classList.add('sim-row-active');

    tr.onclick = (e) => selectFollowedDriver(d.driver_number, d.tla, d.compound, d.tyre_age, d.pit_stop_count, d.team, e);

    const compInfo = getCompoundInfo(d.compound);
    const teamColor = TEAM_COLORS[d.driver_number] || '#a855f7';
    const pits = d.pit_stop_count !== undefined ? d.pit_stop_count : 0;
    const lastLapStr = d.last_lap_time ? formatLapTime(d.last_lap_time) : (d.position <= 3 ? '1:28.452' : '1:29.118');

    // 2026 Regulations: Overtake Mode (MOM - Manual Override Mode) replaces DRS
    const otClass = d.drs_active ? 'ot-active' : 'ot-inactive';
    const otTitle = d.drs_active ? '2026 Manual Override Mode ACTIVE (&lt;1.0s gap)' : 'Manual Override Inactive';

    const pitHtml = d.isPitLap 
      ? `<span class="pit-tag pit-tag-boxing">BOX</span>` 
      : `<span class="pit-tag">${pits}</span>`;

    tr.innerHTML = `
      <td style="font-weight: 700;">
        P${d.position}
        ${d.isSimRow ? '<span class="sim-row-pill">SIM</span>' : ''}
      </td>
      <td style="color: var(--text-muted); font-size: 10px;">${d.driver_number}</td>
      <td>
        <div class="driver-cell">
          <span class="team-stripe" style="background: ${teamColor};"></span>
          <strong>${d.tla}</strong>
        </div>
      </td>
      <td style="font-size: 10px;">${d.gap}</td>
      <td style="font-size: 10px;">${d.interval}</td>
      <td><span class="compound-tag ${compInfo.cssClass}">${compInfo.tag}</span></td>
      <td style="font-size: 10px; color: var(--text-muted);">L${d.tyre_age || 0}</td>
      <td>${pitHtml}</td>
      <td><span class="ot-tag ${otClass}" title="${otTitle}">${d.drs_active ? 'OT' : '---'}</span></td>
      <td style="font-size: 10px; color: #bbbbdd;">${lastLapStr}</td>
    `;

    tbody.appendChild(tr);
  });
}

function selectFollowedDriver(dNo, tla, compound, age, pits, team, evt) {
  followedDriverNo = dNo;
  document.getElementById('followedDriverBadge').innerText = `#${dNo} ${tla}`;
  const teleDriver = document.getElementById('telemetryDriverNo');
  if (teleDriver) teleDriver.value = dNo;

  const teamColor = TEAM_COLORS[dNo] || '#a855f7';
  const compInfo = getCompoundInfo(compound);

  // Update Hero Card in Strategy Tab
  const heroTitle = document.getElementById('heroDriverTitle');
  if (heroTitle) heroTitle.innerText = `#${dNo} ${tla}`;

  const heroStripe = document.getElementById('heroTeamStripe');
  if (heroStripe) heroStripe.style.background = teamColor;

  const driverObj = lerpDriversMap.get(dNo);
  const pos = driverObj ? driverObj.position : 1;
  const heroPos = document.getElementById('heroPosBadge');
  if (heroPos) heroPos.innerText = `P${pos}`;

  const heroTyre = document.getElementById('heroTyreBadge');
  if (heroTyre) {
    heroTyre.innerText = compInfo.tag;
    heroTyre.className = `compound-tag ${compInfo.cssClass}`;
  }

  const heroAge = document.getElementById('heroTyreAge');
  if (heroAge) heroAge.innerText = `L${age || 0}`;

  const heroPits = document.getElementById('heroPitCount');
  if (heroPits) heroPits.innerText = `${pits || 0} PITS`;

  // Update current lap input
  const curLap = driverObj ? (driverObj.current_lap || 20) : 20;
  const curLapInput = document.getElementById('currentLapInput');
  if (curLapInput) curLapInput.value = curLap;

  // Update user strategy default pit lap slider (e.g. current lap + 4)
  const pitRange = document.getElementById('userPitLapRange');
  const pitDisp = document.getElementById('userPitLapDisplay');
  const pitInp = document.getElementById('userPitLapInput');
  if (pitRange && pitDisp && pitInp) {
    const targetLap = Math.min(53, curLap + 4);
    pitRange.value = targetLap;
    pitInp.value = targetLap;
    pitDisp.innerText = `L${targetLap}`;
  }

  // Update adjacent rival labels for context
  const allDrivers = Array.from(lerpDriversMap.values()).sort((a, b) => a.position - b.position);
  const myIdx = allDrivers.findIndex(d => d.driver_number === dNo);
  const rivalAhead = myIdx > 0 ? allDrivers[myIdx - 1] : (allDrivers[1] || { driver_number: 3, tla: 'VER' });
  const rivalBehind = myIdx < allDrivers.length - 1 ? allDrivers[myIdx + 1] : (allDrivers[2] || { driver_number: 16, tla: 'LEC' });

  const r1 = document.getElementById('rival1Label');
  if (r1) r1.innerText = `#${rivalAhead.driver_number} ${rivalAhead.tla}:`;

  const r2 = document.getElementById('rival2Label');
  if (r2) r2.innerText = `#${rivalBehind.driver_number} ${rivalBehind.tla}:`;

  document.querySelectorAll('#timingTowerBody tr').forEach(row => row.classList.remove('followed-row'));
  if (evt && evt.currentTarget) {
    evt.currentTarget.classList.add('followed-row');
  }

  runSimulation();
  runSingleDriverSimulation();
}
