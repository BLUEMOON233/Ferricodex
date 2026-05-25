export type ThemePreference = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

const STORAGE_KEY = "ferricodex.theme";

function safeRead(): ThemePreference {
  if (typeof window === "undefined") {
    return "system";
  }

  try {
    const value = window.localStorage.getItem(STORAGE_KEY);
    if (value === "light" || value === "dark" || value === "system") {
      return value;
    }
  } catch {
    // Ignore unavailable storage and fall through.
  }

  return "system";
}

function safeWrite(value: ThemePreference) {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(STORAGE_KEY, value);
  } catch {
    // Ignore quota or denied storage errors.
  }
}

function systemTheme(): ResolvedTheme {
  if (typeof window === "undefined" || !window.matchMedia) {
    return "light";
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyTheme(resolved: ResolvedTheme) {
  if (typeof document === "undefined") {
    return;
  }

  document.documentElement.dataset.theme = resolved;
}

let preference = $state<ThemePreference>(safeRead());
let systemValue = $state<ResolvedTheme>(systemTheme());

const resolved = $derived<ResolvedTheme>(
  preference === "system" ? systemValue : preference,
);

let initialized = false;

export function initTheme() {
  if (typeof window === "undefined" || initialized) {
    return;
  }

  initialized = true;

  applyTheme(resolved);

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  const handleChange = (event: MediaQueryListEvent) => {
    systemValue = event.matches ? "dark" : "light";
    applyTheme(resolved);
  };

  if (mediaQuery.addEventListener) {
    mediaQuery.addEventListener("change", handleChange);
  } else {
    mediaQuery.addListener(handleChange);
  }
}

export function getThemePreference(): ThemePreference {
  return preference;
}

export function getResolvedTheme(): ResolvedTheme {
  return resolved;
}

export function setThemePreference(next: ThemePreference) {
  preference = next;
  safeWrite(next);
  applyTheme(resolved);
}

export const theme = {
  get preference(): ThemePreference {
    return preference;
  },
  get resolved(): ResolvedTheme {
    return resolved;
  },
  set(next: ThemePreference) {
    setThemePreference(next);
  },
};
