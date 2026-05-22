import { openPath } from "@tauri-apps/plugin-opener";

export function openLocalPath(path: string) {
  return openPath(path);
}
