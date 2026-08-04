import type { CaptureStatus } from "./tauri";

export type CapturePermissionStatus =
  | "granted"
  | "denied"
  | "unknown"
  | "repair-needed";

export type CapturePermissionImpactCode =
  | "settings.capturePermissionImpact.granted"
  | "settings.capturePermissionImpact.denied"
  | "settings.capturePermissionImpact.unknown"
  | "settings.capturePermissionImpact.repairNeeded";

export interface CapturePermissionPresentation {
  status: CapturePermissionStatus;
  messageCode: string;
  messageParams?: CaptureStatus["messageParams"];
  impactCode: CapturePermissionImpactCode;
  devBinaryPath: string | null;
  showPermissionRequest: boolean;
  showRepairAction: boolean;
}

export function deriveCapturePermissionPresentation(
  status: CaptureStatus,
): CapturePermissionPresentation {
  if (status.messageCode === "macosPermissionGrantedNoDisplays") {
    return {
      status: "repair-needed",
      messageCode: status.messageCode,
      messageParams: status.messageParams,
      impactCode: "settings.capturePermissionImpact.repairNeeded",
      devBinaryPath: status.dev_binary_path,
      showPermissionRequest: false,
      showRepairAction: true,
    };
  }

  if (status.screen_capture_granted) {
    return {
      status: "granted",
      messageCode: status.messageCode,
      messageParams: status.messageParams,
      impactCode: "settings.capturePermissionImpact.granted",
      devBinaryPath: null,
      showPermissionRequest: false,
      showRepairAction: false,
    };
  }

  if (status.messageCode === "macosPermissionRequired") {
    return {
      status: "denied",
      messageCode: status.messageCode,
      messageParams: status.messageParams,
      impactCode: "settings.capturePermissionImpact.denied",
      devBinaryPath: status.dev_binary_path,
      showPermissionRequest: true,
      showRepairAction: false,
    };
  }

  return {
    status: "unknown",
    messageCode: status.messageCode,
    messageParams: status.messageParams,
    impactCode: "settings.capturePermissionImpact.unknown",
    devBinaryPath: status.dev_binary_path,
    showPermissionRequest: true,
    showRepairAction: false,
  };
}
