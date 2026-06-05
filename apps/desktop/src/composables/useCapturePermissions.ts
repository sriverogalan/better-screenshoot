import { ref } from "vue";
import { getCaptureStatus, requestScreenCapturePermission } from "../lib/tauri";

export function useCapturePermissions() {
  const permissionMessage = ref<string | null>(null);
  const devBinaryPath = ref<string | null>(null);

  async function checkPermissions() {
    try {
      const status = await getCaptureStatus();
      if (!status.screen_capture_granted) {
        permissionMessage.value = status.message;
        devBinaryPath.value = status.dev_binary_path;
      } else {
        permissionMessage.value = null;
        devBinaryPath.value = null;
      }
    } catch (err) {
      permissionMessage.value =
        err instanceof Error ? err.message : "Could not check permissions";
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
