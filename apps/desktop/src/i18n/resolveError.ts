import type { Composer } from "vue-i18n";

export interface AppErrorPayload {
  code: string;
  detail?: Record<string, string | number>;
}

export function isAppErrorPayload(value: unknown): value is AppErrorPayload {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    typeof (value as AppErrorPayload).code === "string"
  );
}

function parseErrorPayload(value: string): AppErrorPayload | null {
  try {
    const parsed: unknown = JSON.parse(value);
    return isAppErrorPayload(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

export function translateAppError(
  t: Composer["t"],
  payload: string | AppErrorPayload,
): string {
  const errorPayload =
    typeof payload === "string" ? parseErrorPayload(payload) : payload;

  if (!errorPayload) {
    return typeof payload === "string" ? payload : t("errors.generic");
  }

  const key = `errors.${errorPayload.code}`;
  const translated = t(key, errorPayload.detail ?? {});
  return translated === key ? t("errors.generic") : translated;
}

export function translateMessageCode(
  t: Composer["t"],
  code: string,
  params?: Record<string, string | number>,
): string {
  const key = `errors.${code}`;
  const translated = t(key, params ?? {});
  return translated === key ? t("errors.generic") : translated;
}
