const STORAGE_KEY = "ferricodex.zoom";
const MIN_ZOOM = 0.8;
const MAX_ZOOM = 1.5;
const ZOOM_STEP = 0.1;
const DEFAULT_ZOOM = 1.0;
const EPSILON = 0.001;

function clamp(value: number): number {
  if (!Number.isFinite(value)) {
    return DEFAULT_ZOOM;
  }

  const rounded = Math.round(value * 100) / 100;
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, rounded));
}

function safeRead(): number {
  if (typeof window === "undefined") {
    return DEFAULT_ZOOM;
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (raw === null) {
      return DEFAULT_ZOOM;
    }
    return clamp(Number(raw));
  } catch {
    return DEFAULT_ZOOM;
  }
}

function safeWrite(value: number) {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(STORAGE_KEY, value.toFixed(2));
  } catch {
    // Ignore unavailable storage and quota errors.
  }
}

function applyZoom(value: number) {
  if (typeof document === "undefined") {
    return;
  }

  // Expose the level as a CSS variable so body can compensate its layout
  // size with `calc(100vh / var(--zoom-level))`. Without that compensation,
  // `zoom < 1` shrinks visuals but leaves the layout box at viewport size,
  // which renders as a blank strip below the UI.
  document.documentElement.style.setProperty("--zoom-level", String(value));
}

let level = $state<number>(safeRead());
let initialized = false;

export function initZoom() {
  if (initialized || typeof window === "undefined") {
    return;
  }

  initialized = true;
  applyZoom(level);
}

export function setZoom(value: number) {
  const next = clamp(value);
  if (Math.abs(next - level) < EPSILON) {
    return;
  }
  level = next;
  safeWrite(next);
  applyZoom(next);
}

export function zoomIn() {
  setZoom(level + ZOOM_STEP);
}

export function zoomOut() {
  setZoom(level - ZOOM_STEP);
}

export function resetZoom() {
  setZoom(DEFAULT_ZOOM);
}

export const zoom = {
  get level(): number {
    return level;
  },
  get percent(): number {
    return Math.round(level * 100);
  },
  get atMin(): boolean {
    return level <= MIN_ZOOM + EPSILON;
  },
  get atMax(): boolean {
    return level >= MAX_ZOOM - EPSILON;
  },
  get isDefault(): boolean {
    return Math.abs(level - DEFAULT_ZOOM) < EPSILON;
  },
};
