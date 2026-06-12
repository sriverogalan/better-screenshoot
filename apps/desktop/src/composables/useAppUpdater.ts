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
const downloadedBytes = ref(0);
const totalBytes = ref<number | null>(null);
const promptDismissed = ref(false);
const captureSessionActive = ref(false);

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
        errorMessage.value = "Updates are only available in release builds.";
      }
      return false;
    }

    if (phase.value === "checking" || phase.value === "downloading") {
      return false;
    }

    errorMessage.value = null;
    setPhase("checking");

    try {
      const update = await checkForAppUpdate();
      pendingUpdate = update;

      if (!update) {
        setPhase("idle");
        updateSummary.value = null;
        if (!options.silent) {
          errorMessage.value = "You're on the latest version.";
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
      errorMessage.value =
        error instanceof Error ? error.message : "Could not check for updates.";
      return false;
    }
  }

  async function installAvailableUpdate() {
    if (!pendingUpdate || phase.value === "downloading") {
      return;
    }

    if (captureSessionActive.value) {
      errorMessage.value = "Finish the current capture before updating.";
      return;
    }

    errorMessage.value = null;
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
      errorMessage.value =
        error instanceof Error ? error.message : "Could not install the update.";
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
