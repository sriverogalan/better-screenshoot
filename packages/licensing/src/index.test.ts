import { describe, expect, it } from "vitest";
import { TIERS, validateLicenseKey } from "./index";

describe("licensing", () => {
  it("defines open core tiers", () => {
    expect(TIERS.community.billing).toBe("free");
    expect(TIERS.pro.features.length).toBeGreaterThan(0);
  });

  it("validates pro license prefix", async () => {
    const result = await validateLicenseKey("BS-PRO-TEST-123");
    expect(result.valid).toBe(true);
    expect(result.tier).toBe("pro");
  });

  it("rejects invalid keys", async () => {
    const result = await validateLicenseKey("INVALID");
    expect(result.valid).toBe(false);
  });
});
