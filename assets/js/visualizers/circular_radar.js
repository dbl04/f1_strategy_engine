// ==========================================
// 2. ARC-LENGTH CIRCULAR RADAR ENGINE
// ==========================================
class CircularRadarEngine {
  constructor(canvasId) {
    this.canvas = document.getElementById(canvasId);
    this.ctx = this.canvas ? this.canvas.getContext('2d') : null;
  }

  render() {
    if (!this.canvas || !this.ctx || activeVisualizer !== "ring") return;
    const ctx = this.ctx;

    const parentW = this.canvas.parentElement.clientWidth;
    const parentH = this.canvas.parentElement.clientHeight - 34; // DVR bar offset
    if (this.canvas.width !== parentW || this.canvas.height !== parentH) {
      this.canvas.width = parentW;
      this.canvas.height = parentH;
    }

    const width = this.canvas.width;
    const height = this.canvas.height;
    const centerX = width / 2;
    const centerY = height / 2;
    const radarRadius = Math.min(width, height) * 0.38;

    ctx.fillStyle = '#020204';
    ctx.fillRect(0, 0, width, height);

    if (radarRadius <= 0) return;

    // 1. Radar background halo & concentric rings
    ctx.beginPath();
    ctx.arc(centerX, centerY, radarRadius + 20, 0, Math.PI * 2);
    ctx.fillStyle = 'rgba(15, 15, 26, 0.4)';
    ctx.fill();

    ctx.beginPath();
    ctx.arc(centerX, centerY, radarRadius - 20, 0, Math.PI * 2);
    ctx.fillStyle = '#05050a';
    ctx.fill();

    // 2. Main Circular Track Ring (Glow + high-contrast outline)
    ctx.beginPath();
    ctx.arc(centerX, centerY, radarRadius, 0, Math.PI * 2);
    ctx.strokeStyle = '#1a1a2b';
    ctx.lineWidth = 14;
    ctx.stroke();

    ctx.beginPath();
    ctx.arc(centerX, centerY, radarRadius, 0, Math.PI * 2);
    ctx.strokeStyle = '#3b3b55';
    ctx.lineWidth = 4;
    ctx.stroke();

    ctx.beginPath();
    ctx.arc(centerX, centerY, radarRadius, 0, Math.PI * 2);
    ctx.strokeStyle = '#ffffff';
    ctx.lineWidth = 1;
    ctx.stroke();

    // 3. Sector Markers (S1 at 33.3%, S2 at 66.6%, S/F Line at 0%)
    const sectors = [
      { s: 0.0, label: 'START/FINISH', color: '#ffffff', isSF: true },
      { s: 0.333, label: 'SEC 1', color: '#eab308', isSF: false },
      { s: 0.666, label: 'SEC 2', color: '#06b6d4', isSF: false }
    ];

    sectors.forEach(sec => {
      const theta = (2 * Math.PI * sec.s) - (Math.PI / 2);
      const rInner = radarRadius - (sec.isSF ? 14 : 9);
      const rOuter = radarRadius + (sec.isSF ? 14 : 9);

      const x1 = centerX + (rInner * Math.cos(theta));
      const y1 = centerY + (rInner * Math.sin(theta));
      const x2 = centerX + (rOuter * Math.cos(theta));
      const y2 = centerY + (rOuter * Math.sin(theta));

      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.strokeStyle = sec.color;
      ctx.lineWidth = sec.isSF ? 3 : 1.5;
      ctx.stroke();

      // Sector Label
      const rLabel = radarRadius + 22;
      const lx = centerX + (rLabel * Math.cos(theta));
      const ly = centerY + (rLabel * Math.sin(theta));
      ctx.fillStyle = sec.color;
      ctx.font = sec.isSF ? 'bold 10px JetBrains Mono' : 'bold 9px JetBrains Mono';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(sec.label, lx, ly);
    });

    // 4. Center Telemetry HUD Information
    ctx.fillStyle = '#888899';
    ctx.font = '9px JetBrains Mono';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText('CIRCULAR RADAR', centerX, centerY - 18);

    const followedDriver = lerpDriversMap.get(followedDriverNo);
    if (currentTrackStatus === 'SafetyCar') {
      ctx.fillStyle = '#eab308';
      ctx.font = 'bold 12px JetBrains Mono';
      ctx.fillText('⚠️ SAFETY CAR', centerX, centerY);
      ctx.fillStyle = '#ffffff';
      ctx.font = '10px JetBrains Mono';
      ctx.fillText('DELTA PACE ACTIVE', centerX, centerY + 18);
    } else if (currentTrackStatus === 'VirtualSafetyCar') {
      ctx.fillStyle = '#eab308';
      ctx.font = 'bold 12px JetBrains Mono';
      ctx.fillText('🟡 VSC DELTA', centerX, centerY);
      ctx.fillStyle = '#ffffff';
      ctx.font = '10px JetBrains Mono';
      ctx.fillText('REDUCE SPEED 30%', centerX, centerY + 18);
    } else if (currentTrackStatus === 'RedFlag') {
      ctx.fillStyle = '#ef4444';
      ctx.font = 'bold 12px JetBrains Mono';
      ctx.fillText('🔴 RED FLAG', centerX, centerY);
      ctx.fillStyle = '#ffffff';
      ctx.font = '10px JetBrains Mono';
      ctx.fillText('SESSION SUSPENDED', centerX, centerY + 18);
    } else {
      ctx.fillStyle = '#ffffff';
      ctx.font = 'bold 12px JetBrains Mono';
      ctx.fillText(followedDriver ? `#${followedDriver.driver_number} ${followedDriver.tla || getDriverTla(followedDriver)}` : 'LIVE DVR', centerX, centerY);

      ctx.fillStyle = 'var(--color-purple)';
      ctx.font = '10px JetBrains Mono';
      ctx.fillText(followedDriver ? `POS: P${followedDriver.position} | ${followedDriver.gap}` : 'SYNCED', centerX, centerY + 18);
    }

    // 5. Driver Projection on Circular Radar
    lerpDriversMap.forEach(driver => {
      let s = 0.0;
      if (driver.progress !== undefined && driver.progress !== null) {
        s = driver.progress;
      } else if (driver.trackDistance !== undefined && driver.trackTotalDistance) {
        s = driver.trackDistance / driver.trackTotalDistance;
      } else {
        s = getLapProgressForCoords(driver.currentX, driver.currentY);
      }

      s = ((s % 1.0) + 1.0) % 1.0;

      // Compute angle with 12 o'clock start line offset (- Math.PI / 2)
      const theta = (2 * Math.PI * s) - (Math.PI / 2);
      const px = centerX + (radarRadius * Math.cos(theta));
      const py = centerY + (radarRadius * Math.sin(theta));

      const isFollowed = (followedDriverNo === driver.driver_number);
      const isSimCar = (simOverlayState.isActive && simOverlayState.driverNumber === driver.driver_number);
      const color = TEAM_COLORS[driver.driver_number] || '#a855f7';

      if (isSimCar) {
        if (simOverlayState.ghostMode) {
          // Draw Live Ghost Car outline
          ctx.save();
          ctx.beginPath();
          ctx.arc(px, py, 5, 0, Math.PI * 2);
          ctx.strokeStyle = 'rgba(255, 255, 255, 0.45)';
          ctx.setLineDash([2, 2]);
          ctx.lineWidth = 1;
          ctx.stroke();
          ctx.restore();
        }

        const simTheta = (2 * Math.PI * simOverlayState.simulatedProgress) - (Math.PI / 2);
        const inPit = (simOverlayState.status !== "RACING");
        const effectiveR = inPit ? (radarRadius - 14) : radarRadius;
        const drawPx = centerX + (effectiveR * Math.cos(simTheta));
        const drawPy = centerY + (effectiveR * Math.sin(simTheta));
        const simLabelR = effectiveR + (Math.sin(simTheta) > 0 ? 16 : -16);
        const drawLx = centerX + (simLabelR * Math.cos(simTheta));
        const drawLy = centerY + (simLabelR * Math.sin(simTheta));

        const isBoxing = (simOverlayState.status === "IN_PIT_BOX");
        const haloColor = isBoxing ? '#ef4444' : (inPit ? '#f59e0b' : '#06b6d4');

        ctx.beginPath();
        ctx.arc(drawPx, drawPy, 14, 0, Math.PI * 2);
        ctx.strokeStyle = haloColor;
        ctx.lineWidth = 2;
        ctx.stroke();

        ctx.beginPath();
        ctx.arc(drawPx, drawPy, 7, 0, Math.PI * 2);
        ctx.fillStyle = color;
        ctx.fill();

        ctx.beginPath();
        ctx.arc(drawPx, drawPy, 3, 0, Math.PI * 2);
        ctx.fillStyle = '#ffffff';
        ctx.fill();

        ctx.fillStyle = haloColor;
        ctx.font = 'bold 10px JetBrains Mono';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';

        let labelText = `${driver.tla || getDriverTla(driver)} SIM`;
        if (isBoxing) {
          const rem = Math.max(0, simOverlayState.pitStopSecondsRemaining).toFixed(1);
          labelText = `🚨 BOX ${rem}s`;
        } else if (simOverlayState.status === "ENTERING_PITS") {
          labelText = `PIT LIMIT`;
        } else if (simOverlayState.status === "EXITING_PITS") {
          labelText = `PIT OUT`;
        }

        ctx.fillText(labelText, drawLx, drawLy);
        return;
      }

      if (isFollowed) {
        ctx.beginPath();
        ctx.arc(px, py, 14, 0, Math.PI * 2);
        ctx.strokeStyle = '#ffffff';
        ctx.lineWidth = 2;
        ctx.stroke();

        ctx.beginPath();
        ctx.arc(px, py, 10, 0, Math.PI * 2);
        ctx.fillStyle = 'rgba(168, 85, 247, 0.45)';
        ctx.fill();
      }

      ctx.beginPath();
      ctx.arc(px, py, isFollowed ? 7 : 5, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();
      ctx.strokeStyle = '#000000';
      ctx.lineWidth = 1;
      ctx.stroke();

      // Radial label placement for clarity
      const labelR = radarRadius + (Math.sin(theta) > 0 ? 12 : -12);
      const lx = centerX + (labelR * Math.cos(theta));
      const ly = centerY + (labelR * Math.sin(theta));

      ctx.fillStyle = isFollowed ? '#a855f7' : '#ffffff';
      ctx.font = isFollowed ? 'bold 10px JetBrains Mono' : '9px JetBrains Mono';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(`${driver.tla || getDriverTla(driver)}`, lx, ly);
    });
  }
}
