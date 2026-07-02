import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { getCaptureStatus, requestScreenCapturePermission } from "../lib/tauri";
import { translateAppError, translateMessageCode } from "../i18n/resolveError";

/**
 * Builds the human-readable permission message for a given messageCode.
 * The dev-binary hint is appended ONLY when messageCode is 'macosPermissionRequired'
 * (i.e. TCC has NOT been granted). This satisfies spec PD-3.
 *
 * Exported as a pure function so it can be tested independently of Vue/i18n infrastructure.
 */
export function buildPermissionMessage(
  messageCode: string,
  translate: (key: string) => string,
): string {
  let message = translate(messageCode);
  if (messageCode === "macosPermissionRequired") {
    message += translate("macosDevBinaryHint");
  }
  return message;
}

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
