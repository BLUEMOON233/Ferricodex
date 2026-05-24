import { en } from "./i18n/en";
import { zh } from "./i18n/zh";

export type Locale = "en" | "zh";
export type LocalePreference = "auto" | Locale;

const STORAGE_KEY = "codex-history-manager.locale";

const dictionaries: Record<Locale, Record<string, string>> = { en, zh };

function detectSystemLocale(): Locale {
  if (typeof navigator === "undefined") {
    return "en";
  }
  const candidates = [
    ...(Array.isArray(navigator.languages) ? navigator.languages : []),
    navigator.language,
  ];
  for (const value of candidates) {
    if (typeof value === "string" && value.toLowerCase().startsWith("zh")) {
      return "zh";
    }
  }
  return "en";
}

function safeReadPreference(): LocalePreference {
  if (typeof window === "undefined") {
    return "auto";
  }
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (raw === "en" || raw === "zh" || raw === "auto") {
      return raw;
    }
  } catch {
    // ignore
  }
  return "auto";
}

function safeWritePreference(value: LocalePreference) {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.setItem(STORAGE_KEY, value);
  } catch {
    // ignore
  }
}

let preference = $state<LocalePreference>("auto");
let systemLocale = $state<Locale>("en");
let initialized = false;

const activeLocale = $derived<Locale>(preference === "auto" ? systemLocale : preference);

function pluralSuffix(count: number): string {
  return count === 1 ? "_one" : "_other";
}

function interpolate(template: string, params?: Record<string, string | number>): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, key) => {
    const value = params[key];
    return value === undefined || value === null ? match : String(value);
  });
}

export function t(key: string, params?: Record<string, string | number>): string {
  const dict = dictionaries[activeLocale];

  if (params && typeof params.count === "number") {
    const pluralKey = `${key}${pluralSuffix(params.count)}`;
    if (pluralKey in dict) {
      return interpolate(dict[pluralKey], params);
    }
  }

  if (key in dict) {
    return interpolate(dict[key], params);
  }

  const fallback = dictionaries.en[key];
  if (fallback !== undefined) {
    return interpolate(fallback, params);
  }
  return key;
}

export function initI18n() {
  if (initialized || typeof window === "undefined") {
    return;
  }
  initialized = true;
  preference = safeReadPreference();
  systemLocale = detectSystemLocale();

  window.addEventListener("languagechange", () => {
    systemLocale = detectSystemLocale();
  });
}

export const i18n = {
  get preference(): LocalePreference {
    return preference;
  },
  get locale(): Locale {
    return activeLocale;
  },
  get isAuto(): boolean {
    return preference === "auto";
  },
  set(value: LocalePreference) {
    preference = value;
    safeWritePreference(value);
  },
};
