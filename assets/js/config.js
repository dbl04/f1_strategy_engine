// Official F1 2026 Team Colors by driver number
const TEAM_COLORS = {
  1: '#F47600', 3: '#4781D7', 5: '#F50537', 6: '#4781D7', 10: '#00A1E8',
  11: '#909090', 12: '#00D7B6', 14: '#229971', 16: '#ED1131', 18: '#229971',
  23: '#1868DB', 27: '#F50537', 30: '#6C98FF', 31: '#9C9FA2', 41: '#6C98FF',
  43: '#00A1E8', 44: '#ED1131', 55: '#1868DB', 63: '#00D7B6', 77: '#909090',
  81: '#F47600', 87: '#9C9FA2'
};

// Canonical 2026 Grid Driver Metadata Reference
const DRIVER_METADATA = {
  1: { tla: "NOR", name: "Lando Norris", team: "McLaren" },
  3: { tla: "VER", name: "Max Verstappen", team: "Red Bull Racing" },
  5: { tla: "BOR", name: "Gabriel Bortoleto", team: "Audi" },
  6: { tla: "HAD", name: "Isack Hadjar", team: "Red Bull Racing" },
  10: { tla: "GAS", name: "Pierre Gasly", team: "Alpine" },
  11: { tla: "PER", name: "Sergio Perez", team: "Cadillac" },
  12: { tla: "ANT", name: "Kimi Antonelli", team: "Mercedes" },
  14: { tla: "ALO", name: "Fernando Alonso", team: "Aston Martin" },
  16: { tla: "LEC", name: "Charles Leclerc", team: "Ferrari" },
  18: { tla: "STR", name: "Lance Stroll", team: "Aston Martin" },
  23: { tla: "ALB", name: "Alexander Albon", team: "Williams" },
  27: { tla: "HUL", name: "Nico Hulkenberg", team: "Audi" },
  30: { tla: "LAW", name: "Liam Lawson", team: "Racing Bulls" },
  31: { tla: "OCO", name: "Esteban Ocon", team: "Haas F1 Team" },
  41: { tla: "LIN", name: "Arvid Lindblad", team: "Racing Bulls" },
  43: { tla: "COL", name: "Franco Colapinto", team: "Alpine" },
  44: { tla: "HAM", name: "Lewis Hamilton", team: "Ferrari" },
  55: { tla: "SAI", name: "Carlos Sainz", team: "Williams" },
  63: { tla: "RUS", name: "George Russell", team: "Mercedes" },
  77: { tla: "BOT", name: "Valtteri Bottas", team: "Cadillac" },
  81: { tla: "PIA", name: "Oscar Piastri", team: "McLaren" },
  87: { tla: "BEA", name: "Oliver Bearman", team: "Haas F1 Team" },
};

function getDriverTla(driver) {
  if (!driver) return "DRV";
  if (driver.tla && driver.tla !== "undefined") return driver.tla;
  if (driver.name_acronym && driver.name_acronym !== "undefined") return driver.name_acronym;
  const dNo = Number(driver.driver_number);
  if (DRIVER_METADATA[dNo]) return DRIVER_METADATA[dNo].tla;
  if (driver.broadcast_name) return driver.broadcast_name.substring(0, 3).toUpperCase();
  return `#${dNo || '?'}`;
}

function getDriverTeam(driver) {
  if (!driver) return "Formula 1";
  if (driver.team && driver.team !== "undefined") return driver.team;
  if (driver.team_name && driver.team_name !== "undefined") return driver.team_name;
  const dNo = Number(driver.driver_number);
  if (DRIVER_METADATA[dNo]) return DRIVER_METADATA[dNo].team;
  return "Formula 1";
}

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
