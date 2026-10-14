// ==========================================
// 1. 2D CIRCUIT MAP ENGINE (Physical Cartesian)
// ==========================================
class OpenF1RadarEngine {
  constructor(canvasId) {
    this.canvas = document.getElementById(canvasId);
    this.ctx = this.canvas ? this.canvas.getContext('2d') : null;
    this.cachedPath2D = null;
    this.refCoords = [];
    this.screenPositions = [];

    // Interactive mouse hover and click handlers
    if (this.canvas) {
      this.canvas.addEventListener('mousemove', (e) => this.onMouseMove(e));
      this.canvas.addEventListener('mouseleave', () => this.onMouseLeave());
      this.canvas.addEventListener('click', (e) => this.onClick(e));
    }
  }

  onMouseMove(e) {
    const rect = this.canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const tooltip = document.getElementById('mapTooltip');
    if (!tooltip) return;

    let closest = null;
    let minDist = 22;

    for (const sp of this.screenPositions) {
      const dist = Math.hypot(sp.x - mouseX, sp.y - mouseY);
      if (dist < minDist) {
        minDist = dist;
        closest = sp.driver;
      }
    }

    if (closest) {
      const comp = getCompoundInfo(closest.compound);
      tooltip.style.display = 'block';
      tooltip.style.left = `${Math.min(rect.width - 160, mouseX + 12)}px`;
      tooltip.style.top = `${Math.min(rect.height - 95, mouseY - 10)}px`;
      tooltip.innerHTML = `
        <div style="font-weight: 800; color: #fff; margin-bottom: 2px; display: flex; align-items: center; gap: 4px;">
          <span class="team-stripe" style="background: ${TEAM_COLORS[closest.driver_number] || '#a855f7'}; height: 10px;"></span>
          #${closest.driver_number} ${closest.tla} • P${closest.position}
        </div>
        <div style="color: var(--text-muted); font-size: 9px;">${closest.team || 'Formula 1'}</div>
        <div style="margin-top: 4px; display: flex; gap: 4px; align-items: center;">
          <span class="compound-tag ${comp.cssClass}" style="font-size: 8px;">${comp.tag}</span>
          <span>L${closest.tyre_age || 0}</span>
          <span class="pit-tag" style="font-size: 8px;">${closest.pit_stop_count || 0} PITS</span>
        </div>
        <div style="margin-top: 3px; font-size: 9px; color: #aaaacc;">
          GAP: <span style="color: #fff;">${closest.gap}</span> | INT: <span style="color: #fff;">${closest.interval}</span>
        </div>
        ${closest.drs_active ? '<div style="color: var(--color-green); font-size: 8px; font-weight: 800; margin-top: 2px;">⚡ 2026 OVERTAKE MODE (MOM) ACTIVE</div>' : ''}
      `;
      this.canvas.style.cursor = 'pointer';
    } else {
      tooltip.style.display = 'none';
      this.canvas.style.cursor = 'default';
    }
  }

  onMouseLeave() {
    const tooltip = document.getElementById('mapTooltip');
    if (tooltip) tooltip.style.display = 'none';
    if (this.canvas) this.canvas.style.cursor = 'default';
  }

  onClick(e) {
    const rect = this.canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    for (const sp of this.screenPositions) {
      const dist = Math.hypot(sp.x - mouseX, sp.y - mouseY);
      if (dist < 22) {
        selectFollowedDriver(
          sp.driver.driver_number,
          sp.driver.tla,
          sp.driver.compound,
          sp.driver.tyre_age,
          sp.driver.pit_stop_count,
          sp.driver.team
        );
        break;
      }
    }
  }

  async loadReferenceLapMap(sessionKey) {
    this.refCoords = [];
    this.cachedPath2D = null;
    refTrackPoints = [];

    try {
      const resp = await fetch(`/api/track/${sessionKey}`);
      if (resp.ok) {
        const bakedTrack = await resp.json();
        if (Array.isArray(bakedTrack) && bakedTrack.length > 0) {
          this.refCoords = bakedTrack;

          const closedCoords = [...bakedTrack];
          const firstPt = bakedTrack[0];
          const lastPt = bakedTrack[bakedTrack.length - 1];
          const gap = Math.hypot(lastPt.x - firstPt.x, lastPt.y - firstPt.y);
          if (gap > 0.0001) {
            closedCoords.push({ x: firstPt.x, y: firstPt.y });
          }

          let totalDist = 0;
          const pointsWithDist = [];
          for (let i = 0; i < closedCoords.length; i++) {
            if (i === 0) {
              pointsWithDist.push({ x: closedCoords[0].x, y: closedCoords[0].y, dist: 0 });
            } else {
              const dx = closedCoords[i].x - closedCoords[i - 1].x;
              const dy = closedCoords[i].y - closedCoords[i - 1].y;
              const segDist = Math.hypot(dx, dy);
              totalDist += segDist;
              pointsWithDist.push({ x: closedCoords[i].x, y: closedCoords[i].y, dist: totalDist });
            }
          }

          const totalLapDist = totalDist > 0 ? totalDist : 1;
          refTrackPoints = pointsWithDist.map((pt, idx) => ({
            x: pt.x,
            y: pt.y,
            distFromStart: pt.dist,
            progress: (idx === pointsWithDist.length - 1) ? 1.0 : (pt.dist / totalLapDist)
          }));

          this.rebuildPath2D();
        }
      }
    } catch (e) {
      console.error("Failed to load baked track map", e);
    }
    this.render();
  }

  rebuildPath2D() {
    if (!this.canvas || this.refCoords.length === 0) return;
    
    this.canvas.width = this.canvas.parentElement.clientWidth;
    this.canvas.height = this.canvas.parentElement.clientHeight - 34;

    const path = new Path2D();
    this.refCoords.forEach((c, idx) => {
      const { cx, cy } = project(c.x, c.y, this.canvas.width, this.canvas.height);
      if (idx === 0) path.moveTo(cx, cy);
      else path.lineTo(cx, cy);
    });
    path.closePath();

    this.cachedPath2D = path;
  }

  render() {
    if (!this.canvas || !this.ctx || activeVisualizer !== "2d") return;
    const ctx = this.ctx;

    const parentW = this.canvas.parentElement.clientWidth;
    const parentH = this.canvas.parentElement.clientHeight - 34;
    if (this.canvas.width !== parentW || this.canvas.height !== parentH) {
      this.rebuildPath2D();
    }

    ctx.fillStyle = '#020204';
    ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);

    // Track halo & surface
    if (this.cachedPath2D) {
      // Broad glowing underlay
      ctx.strokeStyle = '#12121e';
      ctx.lineWidth = 14;
      ctx.stroke(this.cachedPath2D);

      // Track surface
      ctx.strokeStyle = '#27273a';
      ctx.lineWidth = 7;
      ctx.stroke(this.cachedPath2D);

      // Racing line core
      ctx.strokeStyle = '#606080';
      ctx.lineWidth = 1.5;
      ctx.stroke(this.cachedPath2D);

      // Draw Sector Markers & Start/Finish Line on 2D map
      if (refTrackPoints && refTrackPoints.length > 5) {
        const sfPt = refTrackPoints[0];
        const pSF = project(sfPt.x, sfPt.y, this.canvas.width, this.canvas.height);
        
        // Start/Finish Line marker
        ctx.fillStyle = '#ffffff';
        ctx.beginPath();
        ctx.arc(pSF.cx, pSF.cy, 4, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = 'rgba(255, 255, 255, 0.7)';
        ctx.font = 'bold 8px JetBrains Mono';
        ctx.fillText('S/F', pSF.cx + 6, pSF.cy - 6);

        // Sector 1 marker (~33%)
        const s1Idx = Math.floor(refTrackPoints.length * 0.33);
        const s1Pt = refTrackPoints[s1Idx];
        const pS1 = project(s1Pt.x, s1Pt.y, this.canvas.width, this.canvas.height);
        ctx.fillStyle = '#eab308';
        ctx.beginPath();
        ctx.arc(pS1.cx, pS1.cy, 3, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillText('S1', pS1.cx + 5, pS1.cy - 5);

        // Sector 2 marker (~66%)
        const s2Idx = Math.floor(refTrackPoints.length * 0.66);
        const s2Pt = refTrackPoints[s2Idx];
        const pS2 = project(s2Pt.x, s2Pt.y, this.canvas.width, this.canvas.height);
        ctx.fillStyle = '#06b6d4';
        ctx.beginPath();
        ctx.arc(pS2.cx, pS2.cy, 3, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillText('S2', pS2.cx + 5, pS2.cy - 5);
      }
    }

    const padding = Math.min(this.canvas.width, this.canvas.height) * 0.05;
    const usableSize = Math.min(this.canvas.width, this.canvas.height) - (padding * 2);
    const offsetX = (this.canvas.width - usableSize) / 2;
    const offsetY = (this.canvas.height - usableSize) / 2;

    this.screenPositions = [];

    lerpDriversMap.forEach(driver => {
      const px = offsetX + (driver.currentX * usableSize);
      const py = this.canvas.height - (offsetY + (driver.currentY * usableSize));

      const isFollowed = (followedDriverNo === driver.driver_number);
      const isSimCar = (simOverlayState.isActive && simOverlayState.driverNumber === driver.driver_number);
      const color = TEAM_COLORS[driver.driver_number] || '#a855f7';

      if (isSimCar) {
        if (simOverlayState.ghostMode) {
          // Draw Live Ghost Car outline on 2D map
          ctx.save();
          ctx.beginPath();
          ctx.arc(px, py, 6, 0, Math.PI * 2);
          ctx.strokeStyle = 'rgba(255, 255, 255, 0.45)';
          ctx.setLineDash([2, 2]);
          ctx.lineWidth = 1.5;
          ctx.stroke();
          ctx.restore();

          // Ghost Car Label
          ctx.fillStyle = 'rgba(255, 255, 255, 0.45)';
          ctx.font = '8px JetBrains Mono';
          ctx.fillText(`${driver.tla} (LIVE)`, px + 8, py - 6);
        }

        // Map simulated progress to track coordinates
        const simNorm = getCoordsForProgress(simOverlayState.simulatedProgress);
        const simProj = project(simNorm.x, simNorm.y, this.canvas.width, this.canvas.height);
        const drawPx = simProj.cx;
        const drawPy = simProj.cy;

        this.screenPositions.push({ x: drawPx, y: drawPy, driver });

        // Glowing cyan halo for simulated car
        const now = Date.now();
        const pulseR = 14 + Math.sin(now / 150) * 4;
        ctx.beginPath();
        ctx.arc(drawPx, drawPy, pulseR, 0, Math.PI * 2);
        ctx.strokeStyle = 'rgba(6, 182, 212, 0.65)';
        ctx.lineWidth = 2;
        ctx.stroke();

        ctx.beginPath();
        ctx.arc(drawPx, drawPy, 10, 0, Math.PI * 2);
        ctx.fillStyle = 'rgba(6, 182, 212, 0.35)';
        ctx.fill();

        // Outer dot with team color
        ctx.beginPath();
        ctx.arc(drawPx, drawPy, 8, 0, Math.PI * 2);
        ctx.fillStyle = color;
        ctx.fill();

        // Core white dot
        ctx.beginPath();
        ctx.arc(drawPx, drawPy, 3.5, 0, Math.PI * 2);
        ctx.fillStyle = '#ffffff';
        ctx.fill();

        // Acronym SIM Pill Badge
        const badgeX = drawPx + 8;
        const badgeY = drawPy - 7;
        const isPitStop = simOverlayState.isCurrentlyBoxing || (simOverlayState.timeline.length > 0 && simOverlayState.timeline[simOverlayState.activeTimelineIndex] && simOverlayState.timeline[simOverlayState.activeTimelineIndex].is_pit_lap);

        if (isPitStop) {
          ctx.fillStyle = 'rgba(239, 68, 68, 0.85)';
          ctx.fillRect(badgeX, badgeY, 68, 12);
          ctx.strokeStyle = '#ef4444';
          ctx.lineWidth = 1;
          ctx.strokeRect(badgeX, badgeY, 68, 12);

          ctx.fillStyle = '#ffffff';
          ctx.font = 'bold 8px JetBrains Mono';
          ctx.fillText(`BOXING 2.4s`, badgeX + 3, badgeY + 9);
        } else {
          ctx.fillStyle = '#061a24';
          ctx.fillRect(badgeX, badgeY, 44, 12);
          ctx.strokeStyle = '#06b6d4';
          ctx.lineWidth = 1;
          ctx.strokeRect(badgeX, badgeY, 44, 12);

          ctx.fillStyle = '#06b6d4';
          ctx.font = 'bold 9px JetBrains Mono';
          ctx.fillText(`${driver.tla} SIM`, badgeX + 3, badgeY + 9);
        }
        return;
      }

      this.screenPositions.push({ x: px, y: py, driver });

      // Pulsating halo for followed driver
      if (isFollowed) {
        const now = Date.now();
        const pulseR = 14 + Math.sin(now / 180) * 4;
        const pulseAlpha = 0.35 + Math.sin(now / 180) * 0.15;

        ctx.beginPath();
        ctx.arc(px, py, pulseR, 0, Math.PI * 2);
        ctx.strokeStyle = `rgba(168, 85, 247, ${pulseAlpha})`;
        ctx.lineWidth = 2;
        ctx.stroke();

        ctx.beginPath();
        ctx.arc(px, py, 11, 0, Math.PI * 2);
        ctx.fillStyle = 'rgba(168, 85, 247, 0.45)';
        ctx.fill();
      }

      // Outer colored glow ring
      ctx.beginPath();
      ctx.arc(px, py, isFollowed ? 8 : 6, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();

      // Inner white core
      ctx.beginPath();
      ctx.arc(px, py, isFollowed ? 4 : 3, 0, Math.PI * 2);
      ctx.fillStyle = '#ffffff';
      ctx.fill();

      // Acronym Pill Badge
      const badgeX = px + 8;
      const badgeY = py - 7;
      ctx.fillStyle = isFollowed ? '#160824' : '#08080f';
      ctx.fillRect(badgeX, badgeY, 26, 12);
      ctx.strokeStyle = isFollowed ? '#a855f7' : '#222238';
      ctx.lineWidth = 1;
      ctx.strokeRect(badgeX, badgeY, 26, 12);

      ctx.fillStyle = isFollowed ? '#a855f7' : '#ffffff';
      ctx.font = 'bold 9px JetBrains Mono';
      ctx.fillText(`${driver.tla}`, badgeX + 3, badgeY + 9);

      // 2026 Overtake Mode Pip (green indicator when Manual Override Mode active)
      if (driver.drs_active) {
        ctx.beginPath();
        ctx.arc(px, py - 9, 2.5, 0, Math.PI * 2);
        ctx.fillStyle = '#22c55e';
        ctx.fill();
      }
    });
  }
}
