import { invoke } from "@tauri-apps/api/core";

export function openLocalPath(path: string) {
  return invoke<void>("open_local_path", { path });
}
