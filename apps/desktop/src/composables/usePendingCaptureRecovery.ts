import { onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  openPendingCaptureInEditor,
  peekPendingCapture,
  type SavedCapture,
} from "../lib/tauri";
import { translateAppError } from "../i18n/resolveError";

const EDITOR_OPEN_GRACE_MS = 2500;

export function usePendingCaptureRecovery() {
  const { t } = useI18n();
  const pendingCapture = ref<SavedCapture | null>(null);
  const recoveryBusy = ref(false);
  const recoveryError = ref<string | null>(null);

  async function refreshPendingCapture() {
    try {
      pendingCapture.value = await peekPendingCapture();
    } catch {
      pendingCapture.value = null;
    }
  }

  async function openPendingInEditor() {
    if (recoveryBusy.value) return;

    recoveryBusy.value = true;
    recoveryError.value = null;
    try {
      await openPendingCaptureInEditor();
      await refreshPendingCapture();
      if (!pendingCapture.value) {
        recoveryError.value = null;
      }
    } catch (err) {
      recoveryError.value =
        err instanceof Error
          ? translateAppError(t, err.message)
          : t("errors.openEditorFailed");
      await refreshPendingCapture();
    } finally {
      recoveryBusy.value = false;
    }
  }

  let unlisteners: UnlistenFn[] = [];
  let pendingBannerTimer: number | undefined;
  let editorOpenedSinceCapture = false;

  onMounted(async () => {
    await refreshPendingCapture();
    unlisteners = await Promise.all([
      listen("capture-error", () => {
        void refreshPendingCapture();
      }),
      listen("capture-complete", () => {
        editorOpenedSinceCapture = false;
        if (pendingBannerTimer !== undefined) {
          clearTimeout(pendingBannerTimer);
        }
        pendingBannerTimer = window.setTimeout(() => {
          pendingBannerTimer = undefined;
          if (!editorOpenedSinceCapture) {
            void refreshPendingCapture();
          }
        }, EDITOR_OPEN_GRACE_MS);
      }),
      listen("editor-opened", () => {
        editorOpenedSinceCapture = true;
        if (pendingBannerTimer !== undefined) {
          clearTimeout(pendingBannerTimer);
          pendingBannerTimer = undefined;
        }
        pendingCapture.value = null;
        recoveryError.value = null;
      }),
    ]);
  });

  onUnmounted(() => {
    if (pendingBannerTimer !== undefined) {
      clearTimeout(pendingBannerTimer);
    }
    unlisteners.forEach((unlisten) => unlisten());
  });

  return {
    pendingCapture,
    recoveryBusy,
    recoveryError,
    refreshPendingCapture,
    openPendingInEditor,
  };
}
