// Official F1 2026 Team Colors by driver number
const TEAM_COLORS = {
  1: '#F47600', 3: '#4781D7', 5: '#F50537', 6: '#4781D7', 10: '#00A1E8',
  11: '#909090', 12: '#00D7B6', 14: '#229971', 16: '#ED1131', 18: '#229971',
  23: '#1868DB', 27: '#F50537', 30: '#6C98FF', 31: '#9C9FA2', 41: '#6C98FF',
  43: '#00A1E8', 44: '#ED1131', 55: '#1868DB', 63: '#00D7B6', 77: '#909090',
  81: '#F47600', 87: '#9C9FA2'
};

// Helper: Map (x,y) to normalized track progress s in [0.0, 1.0]
function getLapProgressForCoords(x, y) {
  if (!refTrackPoints || refTrackPoints.length === 0) return 0;
  let minSqDist = Infinity;
  let closestProgress = 0;
  for (let i = 0; i < refTrackPoints.length; i++) {
    const pt = refTrackPoints[i];
    const dx = pt.x - x;
    const dy = pt.y - y;
    const sqDist = dx * dx + dy * dy;
    if (sqDist < minSqDist) {
      minSqDist = sqDist;
      closestProgress = pt.progress;
    }
  }
  return closestProgress;
}

// Helper: Map progress s in [0.0, 1.0] to interpolated (x,y) coordinates on circuit with continuous closed loop
function getCoordsForProgress(s) {
  if (!refTrackPoints || refTrackPoints.length === 0) return { x: 0.5, y: 0.5 };
  const normS = ((s % 1.0) + 1.0) % 1.0;
  const total = refTrackPoints.length;

  let low = 0;
  let high = total - 1;

  while (low <= high) {
    const mid = (low + high) >> 1;
    if (refTrackPoints[mid].progress <= normS) {
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }

  const idx1 = Math.max(0, Math.min(total - 1, low - 1));
  const idx2 = Math.min(total - 1, idx1 + 1);

  const pt1 = refTrackPoints[idx1];
  const pt2 = refTrackPoints[idx2];
  const segLen = pt2.progress - pt1.progress;
  const frac = (segLen > 0.000001) ? (normS - pt1.progress) / segLen : 0;

  return {
    x: pt1.x + (pt2.x - pt1.x) * frac,
    y: pt1.y + (pt2.y - pt1.y) * frac
  };
}

// Universal Linear Projection for 2D Circuit Map (0.0 - 1.0 normalized coords)
function project(nx, ny, canvasWidth, canvasHeight) {
  const padding = Math.min(canvasWidth, canvasHeight) * 0.05;
  const usableSize = Math.min(canvasWidth, canvasHeight) - (padding * 2);
  const offsetX = (canvasWidth - usableSize) / 2;
  const offsetY = (canvasHeight - usableSize) / 2;

  return {
    cx: offsetX + (nx * usableSize),
    cy: canvasHeight - (offsetY + (ny * usableSize))
  };
}

function formatTime(totalSeconds) {
  const s = Math.floor(totalSeconds % 60);
  const m = Math.floor((totalSeconds / 60) % 60);
  const h = Math.floor(totalSeconds / 3600);
  const pad = (n) => n.toString().padStart(2, '0');
  return `T + ${pad(h)}:${pad(m)}:${pad(s)}`;
}

function formatLapTime(totalSeconds) {
  if (!totalSeconds || isNaN(totalSeconds)) return "-";
  const m = Math.floor(totalSeconds / 60);
  const s = (totalSeconds % 60).toFixed(3);
  return `${m}:${s.padStart(6, '0')}`;
}

function getCompoundInfo(compStr) {
  if (!compStr) return { tag: 'MED', cssClass: 'comp-medium', full: 'MEDIUM' };
  const upper = compStr.toString().trim().toUpperCase();
  if (upper.includes('SOFT') || upper === 'S') return { tag: 'SOFT', cssClass: 'comp-soft', full: 'SOFT' };
  if (upper.includes('MED') || upper === 'M') return { tag: 'MED', cssClass: 'comp-medium', full: 'MEDIUM' };
  if (upper.includes('HARD') || upper === 'H') return { tag: 'HARD', cssClass: 'comp-hard', full: 'HARD' };
  if (upper.includes('INTER') || upper === 'I') return { tag: 'INT', cssClass: 'comp-inter', full: 'INTERMEDIATE' };
  if (upper.includes('WET') || upper === 'W') return { tag: 'WET', cssClass: 'comp-wet', full: 'WET' };
  return { tag: upper.substring(0, 4), cssClass: 'comp-medium', full: upper };
}
