export type Platform = "mac" | "windows" | "linux" | "other";

function detectPlatform(): Platform {
  if (typeof navigator === "undefined") {
    return "other";
  }

  const uaData = (navigator as Navigator & {
    userAgentData?: { platform?: string };
  }).userAgentData;

  const raw = (uaData?.platform ?? navigator.platform ?? navigator.userAgent ?? "").toLowerCase();

  if (raw.includes("mac") || raw.includes("darwin") || raw.includes("iphone") || raw.includes("ipad")) {
    return "mac";
  }
  if (raw.includes("win")) {
    return "windows";
  }
  if (raw.includes("linux") || raw.includes("x11")) {
    return "linux";
  }
  return "other";
}

export const platform: Platform = detectPlatform();

export const isMac = platform === "mac";

export const modKeyLabel = isMac ? "⌘" : "Ctrl";

export const altKeyLabel = isMac ? "⌥" : "Alt";

export const shiftKeyLabel = isMac ? "⇧" : "Shift";
