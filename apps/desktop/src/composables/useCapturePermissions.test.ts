import { describe, expect, it } from "vitest";
import { buildPermissionMessage } from "./useCapturePermissions";

// 8.4: Dev-binary hint MUST NOT appear when messageCode !== 'macosPermissionRequired'

describe("buildPermissionMessage", () => {
  const translate = (key: string) => {
    const map: Record<string, string> = {
      "macosPermissionRequired": "macOS does not allow screen capture.",
      "macosPermissionGrantedNoDisplays": "Screen Recording allowed but no displays found.",
      "displaysDetected": "Display detected.",
      "macosDevBinaryHint": " Also authorize the dev binary.",
    };
    return map[key] ?? key;
  };

  it("appends dev binary hint only when messageCode is macosPermissionRequired", () => {
    const result = buildPermissionMessage("macosPermissionRequired", translate);
    expect(result).toContain("Also authorize the dev binary.");
  });

  it("does NOT append dev binary hint when messageCode is macosPermissionGrantedNoDisplays", () => {
    const result = buildPermissionMessage("macosPermissionGrantedNoDisplays", translate);
    expect(result).not.toContain("Also authorize the dev binary.");
    expect(result).toContain("Screen Recording allowed but no displays found.");
  });

  it("does NOT append dev binary hint when messageCode is displaysDetected", () => {
    const result = buildPermissionMessage("displaysDetected", translate);
    expect(result).not.toContain("Also authorize the dev binary.");
  });
});
