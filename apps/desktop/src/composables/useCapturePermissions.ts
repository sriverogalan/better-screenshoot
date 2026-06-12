import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { getCaptureStatus, requestScreenCapturePermission } from "../lib/tauri";
import { translateAppError, translateMessageCode } from "../i18n/resolveError";

export function useCapturePermissions() {
  const { t } = useI18n();
  const permissionMessage = ref<string | null>(null);
  const devBinaryPath = ref<string | null>(null);

  async function checkPermissions() {
    try {
      const status = await getCaptureStatus();
      if (!status.screen_capture_granted) {
        let message = translateMessageCode(
          t,
          status.messageCode,
          status.messageParams ?? undefined,
        );
        if (status.messageCode === "macosPermissionRequired") {
          message += t("errors.macosDevBinaryHint");
        }
        permissionMessage.value = message;
        devBinaryPath.value = status.dev_binary_path;
      } else {
        permissionMessage.value = null;
        devBinaryPath.value = null;
      }
    } catch (err) {
      permissionMessage.value =
        err instanceof Error
          ? translateAppError(t, err.message)
          : t("errors.checkPermissionsFailed");
    }
  }

  async function requestPermission() {
    await requestScreenCapturePermission();
    await checkPermissions();
  }

  return {
    permissionMessage,
    devBinaryPath,
    checkPermissions,
    requestPermission,
  };
}
