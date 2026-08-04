import { createI18n } from "vue-i18n";
import type { AppLocale } from "@better-screenshoot/shared-types";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import en from "../locales/en.json";
import es from "../locales/es.json";
import fr from "../locales/fr.json";
import de from "../locales/de.json";
import pt from "../locales/pt.json";
import it from "../locales/it.json";
import "./types";

export const SUPPORTED_LOCALES: AppLocale[] = ["en", "es", "fr", "de", "pt", "it"];
export const FALLBACK_LOCALE: AppLocale = "en";

const messages = { en, es, fr, de, pt, it } as const;

export function detectSystemLocale(): AppLocale {
  const language = navigator.language.toLowerCase();
  if (language.startsWith("es")) return "es";
  if (language.startsWith("fr")) return "fr";
  if (language.startsWith("de")) return "de";
  if (language.startsWith("pt")) return "pt";
  if (language.startsWith("it")) return "it";
  return "en";
}

export function normalizeLocale(value: string | undefined): AppLocale {
  if (value && SUPPORTED_LOCALES.includes(value as AppLocale)) {
    return value as AppLocale;
  }
  return FALLBACK_LOCALE;
}

export const i18n = createI18n({
  legacy: false,
  locale: FALLBACK_LOCALE,
  fallbackLocale: FALLBACK_LOCALE,
  messages,
});

function applyDocumentLocale(locale: AppLocale) {
  document.documentElement.lang = locale;
}

export async function setLocale(locale: AppLocale) {
  i18n.global.locale.value = locale;
  applyDocumentLocale(locale);

  try {
    await invoke("update_tray_tooltip");
  } catch {
    // Tray may be unavailable in tests or non-Tauri contexts.
  }

  await emit("locale-changed", locale);
}

export function initLocaleFromSettings(locale: AppLocale) {
  i18n.global.locale.value = locale;
  applyDocumentLocale(locale);
}

export function setupLocaleListener() {
  return listen<AppLocale>("locale-changed", (event) => {
    if (event.payload) {
      initLocaleFromSettings(normalizeLocale(event.payload));
    }
  });
}
