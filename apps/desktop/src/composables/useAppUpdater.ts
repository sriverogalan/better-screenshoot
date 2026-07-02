import { computed, ref } from "vue";
import type { Update } from "@tauri-apps/plugin-updater";
import {
  checkForAppUpdate,
  downloadAndInstallAppUpdate,
  getAppVersion,
  isUpdaterSupported,
  summarizeUpdate,
  type UpdatePhase,
  type UpdateSummary,
} from "../lib/updater";

const phase = ref<UpdatePhase>("idle");
const currentVersion = ref<string | null>(null);
const updateSummary = ref<UpdateSummary | null>(null);
const errorMessage = ref<string | null>(null);
const statusCode = ref<AppUpdaterStatusCode | null>(null);
const downloadedBytes = ref(0);
const totalBytes = ref<number | null>(null);
const promptDismissed = ref(false);
const captureSessionActive = ref(false);

export type AppUpdaterStatusCode =
  | "unsupported"
  | "latest"
  | "checkFailed"
  | "captureActive"
  | "installFailed";

let pendingUpdate: Update | null = null;
let startupCheckScheduled = false;
let accumulatedBytes = 0;

const updateAvailable = computed(
  () => phase.value === "available" && updateSummary.value !== null,
);

const showPrompt = computed(
  () =>
    updateAvailable.value &&
    !promptDismissed.value &&
    !captureSessionActive.value,
);

function resetProgress() {
  downloadedBytes.value = 0;
  totalBytes.value = null;
  accumulatedBytes = 0;
}

function setPhase(next: UpdatePhase) {
  phase.value = next;
}

export function useAppUpdater() {
  async function loadCurrentVersion() {
    try {
      currentVersion.value = await getAppVersion();
    } catch {
      currentVersion.value = null;
    }
  }

  async function checkForUpdates(options: { silent?: boolean } = {}) {
    if (!isUpdaterSupported()) {
      if (!options.silent) {
        statusCode.value = "unsupported";
        errorMessage.value = null;
      }
      return false;
    }

    if (phase.value === "checking" || phase.value === "downloading") {
      return false;
    }

    errorMessage.value = null;
    statusCode.value = null;
    setPhase("checking");

    try {
      const update = await checkForAppUpdate();
      pendingUpdate = update;

      if (!update) {
        setPhase("idle");
        updateSummary.value = null;
        if (!options.silent) {
          statusCode.value = "latest";
        }
        return false;
      }

      updateSummary.value = summarizeUpdate(update);
      promptDismissed.value = false;
      setPhase("available");
      return true;
    } catch (error) {
      pendingUpdate = null;
      updateSummary.value = null;
      setPhase("error");
      statusCode.value = error instanceof Error ? null : "checkFailed";
      errorMessage.value = error instanceof Error ? error.message : null;
      return false;
    }
  }

  async function installAvailableUpdate() {
    if (!pendingUpdate || phase.value === "downloading") {
      return;
    }

    if (captureSessionActive.value) {
      statusCode.value = "captureActive";
      errorMessage.value = null;
      return;
    }

    errorMessage.value = null;
    statusCode.value = null;
    resetProgress();
    setPhase("downloading");

    try {
      await downloadAndInstallAppUpdate(pendingUpdate, (progress) => {
        if (progress.totalBytes !== null) {
          totalBytes.value = progress.totalBytes;
          downloadedBytes.value = 0;
          accumulatedBytes = 0;
          return;
        }

        accumulatedBytes += progress.downloadedBytes;
        downloadedBytes.value = accumulatedBytes;
      });
      setPhase("installing");
    } catch (error) {
      setPhase("error");
      statusCode.value = error instanceof Error ? null : "installFailed";
      errorMessage.value = error instanceof Error ? error.message : null;
    }
  }

  function dismissPrompt() {
    promptDismissed.value = true;
  }

  function setCaptureSessionActive(active: boolean) {
    captureSessionActive.value = active;
  }

  function scheduleStartupCheck(delayMs = 5000) {
    if (startupCheckScheduled || !isUpdaterSupported()) {
      return;
    }

    startupCheckScheduled = true;

    window.setTimeout(() => {
      if (captureSessionActive.value) {
        return;
      }

      void checkForUpdates({ silent: true });
    }, delayMs);
  }

  return {
    phase,
    currentVersion,
    updateSummary,
    errorMessage,
    statusCode,
    downloadedBytes,
    totalBytes,
    updateAvailable,
    showPrompt,
    loadCurrentVersion,
    checkForUpdates,
    installAvailableUpdate,
    dismissPrompt,
    setCaptureSessionActive,
    scheduleStartupCheck,
  };
}
