export function formatDate(timestampMs: number) {
  return new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(timestampMs));
}

export function formatSize(bytes: number | null | undefined) {
  if (bytes === null || bytes === undefined) {
    return "Unknown";
  }

  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  const digits = value >= 10 || unitIndex === 0 ? 0 : 1;

  return `${value.toFixed(digits)} ${units[unitIndex]}`;
}

export function formatCount(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return "Unknown";
  }

  return new Intl.NumberFormat().format(value);
}
