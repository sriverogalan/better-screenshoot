import { describe, expect, it } from "vitest";
import { isMacOsUserAgent } from "./editor-window";

describe("isMacOsUserAgent", () => {
  it("detects macOS user agents", () => {
    expect(
      isMacOsUserAgent(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
      ),
    ).toBe(true);
  });

  it("ignores non-macOS user agents", () => {
    expect(
      isMacOsUserAgent(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
      ),
    ).toBe(false);
  });
});
