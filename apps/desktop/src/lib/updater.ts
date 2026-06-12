import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

export type UpdatePhase =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "error";

export interface UpdateSummary {
  version: string;
  notes: string | null;
  date: string | null;
}

export interface UpdateProgress {
  downloadedBytes: number;
  totalBytes: number | null;
}

export async function getAppVersion(): Promise<string> {
  return getVersion();
}

export function isUpdaterSupported(): boolean {
  return import.meta.env.PROD;
}

export function summarizeUpdate(update: Update): UpdateSummary {
  return {
    version: update.version,
    notes: update.body ?? null,
    date: update.date ?? null,
  };
}

export async function checkForAppUpdate(): Promise<Update | null> {
  if (!isUpdaterSupported()) {
    return null;
  }

  return check();
}

export async function downloadAndInstallAppUpdate(
  update: Update,
  onProgress?: (progress: UpdateProgress) => void,
): Promise<void> {
  await update.downloadAndInstall((event: DownloadEvent) => {
    if (!onProgress) {
      return;
    }

    switch (event.event) {
      case "Started":
        onProgress({
          downloadedBytes: 0,
          totalBytes: event.data.contentLength ?? null,
        });
        break;
      case "Progress":
        onProgress({
          downloadedBytes: event.data.chunkLength,
          totalBytes: null,
        });
        break;
      case "Finished":
        break;
    }
  });

  await relaunch();
}
