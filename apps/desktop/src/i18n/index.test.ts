import { describe, expect, it } from "vitest";
import { detectSystemLocale, normalizeLocale } from "./index";

describe("i18n locale helpers", () => {
  it("normalizes supported locales", () => {
    expect(normalizeLocale("es")).toBe("es");
    expect(normalizeLocale("fr")).toBe("fr");
    expect(normalizeLocale("unknown")).toBe("en");
  });

  it("detects spanish from navigator language", () => {
    const original = navigator.language;
    Object.defineProperty(navigator, "language", {
      configurable: true,
      value: "es-ES",
    });
    expect(detectSystemLocale()).toBe("es");
    Object.defineProperty(navigator, "language", {
      configurable: true,
      value: original,
    });
  });
});
