<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  DEFAULT_HOTKEYS,
  SYSTEM_REPLACEMENT_HOTKEYS,
  type AppLocale,
  type HotkeyConfig,
  type SystemCaptureMode,
} from "@better-screenshoot/shared-types";
import { useSettingsStore } from "../stores/settings";
import SystemScreenshotPermissionDialog from "../components/settings/SystemScreenshotPermissionDialog.vue";
import AppUpdateSection from "../components/settings/AppUpdateSection.vue";
import {
  getSystemCaptureStatus,
  setSystemCaptureMode,
  type SystemCaptureStatus,
} from "../lib/tauri";
import { formatHotkey } from "../lib/format-hotkey";
import PendingCaptureBanner from "../components/PendingCaptureBanner.vue";
import { SUPPORTED_LOCALES, setLocale } from "../i18n";
import { translateAppError, translateMessageCode } from "../i18n/resolveError";
import { systemShortcutLabelKey } from "../lib/system-shortcut-labels";

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const settingsStore = useSettingsStore();

const hotkeyFields = computed(() => [
  {
    key: "capture_area" as const,
    label: t("settings.hotkeys.captureArea"),
    hint: t("settings.hotkeys.captureAreaHint"),
  },
  { key: "capture_screen" as const, label: t("settings.hotkeys.captureScreen") },
  { key: "capture_window" as const, label: t("settings.hotkeys.captureWindow") },
  { key: "open_history" as const, label: t("settings.hotkeys.openHistory") },
]);

const captureHotkeyKeys: Array<keyof HotkeyConfig> = [
  "capture_area",
  "capture_screen",
  "capture_window",
];

const settings = computed(() => settingsStore.settings);
const systemMessage = ref<string | null>(null);
const systemSuccess = ref<string | null>(null);
const systemBusy = ref(false);
const showReplaceDialog = ref(false);
const captureStatus = ref<SystemCaptureStatus | null>(null);

const languageOptions = computed(() =>
  SUPPORTED_LOCALES.map((locale) => ({
    value: locale,
    label: t(`settings.languageOptions.${locale}`),
  })),
);

const independentHotkeyPreview = computed(() =>
  [
    `${formatHotkey(DEFAULT_HOTKEYS.capture_area)} ${t("common.region")}`,
    `${formatHotkey(DEFAULT_HOTKEYS.capture_screen)} ${t("common.screen")}`,
    `${formatHotkey(DEFAULT_HOTKEYS.capture_window)} ${t("common.window")}`,
  ].join(" · "),
);

const replacementHotkeyPreview = computed(() =>
  [
    `${formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_screen)} ${t("common.screen")}`,
    `${formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_area)} ${t("common.region")}`,
    `${formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_window)} ${t("common.window")}`,
  ].join(" · "),
);

const currentMode = computed(() => settings.value.system_capture_mode);
const isReplaceMode = computed(() => currentMode.value === "replace_system");
const driftDetected = computed(() => captureStatus.value?.drift_detected ?? false);
const driftMessage = computed(() => {
  const code = captureStatus.value?.messageCode;
  return code ? translateMessageCode(t, code) : null;
});

function isCaptureHotkeyLocked(key: keyof HotkeyConfig) {
  return isReplaceMode.value && captureHotkeyKeys.includes(key);
}

function shortcutLabel(id: number, fallback: string) {
  const key = systemShortcutLabelKey(id);
  return key ? t(key) : fallback;
}

function managedShortcut(key: keyof HotkeyConfig) {
  if (key === "capture_screen") return "⌘⇧3";
  if (key === "capture_area") return "⌘⇧4";
  return "⌘⇧5";
}

async function updateField<K extends keyof typeof settings.value>(
  key: K,
  value: (typeof settings.value)[K],
) {
  await settingsStore.save({ ...settings.value, [key]: value });
}

async function updateLocale(locale: AppLocale) {
  await settingsStore.save({ ...settings.value, locale });
  await setLocale(locale);
}

const isOnOnboardingRoute = computed(() => route.path === "/onboarding");

async function runSetupWizardAgain() {
  await settingsStore.save({ ...settings.value, onboarding_completed: false });
  await router.push("/onboarding");
}

async function updateHotkey(
  key: keyof typeof settings.value.hotkeys,
  value: string,
) {
  await settingsStore.save({
    ...settings.value,
    hotkeys: { ...settings.value.hotkeys, [key]: value },
  });
}

async function loadCaptureStatus() {
  try {
    captureStatus.value = await getSystemCaptureStatus();
  } catch (err) {
    systemMessage.value =
      err instanceof Error
        ? translateAppError(t, err.message)
        : t("errors.checkCaptureModeFailed");
  }
}

async function applyCaptureMode(mode: SystemCaptureMode) {
  systemBusy.value = true;
  systemMessage.value = null;
  systemSuccess.value = null;
  try {
    const result = await setSystemCaptureMode(mode);
    settingsStore.settings = result.settings;
    captureStatus.value = result.status;
    systemSuccess.value = translateMessageCode(t, result.messageCode);
  } catch (err) {
    systemMessage.value =
      err instanceof Error
        ? translateAppError(t, err.message)
        : t("errors.changeCaptureModeFailed");
    await loadCaptureStatus();
  } finally {
    systemBusy.value = false;
  }
}

async function onModeChange(mode: SystemCaptureMode) {
  if (mode === currentMode.value || systemBusy.value) return;

  if (mode === "replace_system") {
    await loadCaptureStatus();
    showReplaceDialog.value = true;
    return;
  }

  await applyCaptureMode("independent");
}

async function confirmReplaceMode() {
  await applyCaptureMode("replace_system");
  showReplaceDialog.value = false;
}

async function restoreSystemCaptures() {
  await applyCaptureMode("independent");
}

async function repairDrift() {
  await applyCaptureMode("independent");
}

let unlisteners: UnlistenFn[] = [];

onMounted(async () => {
  await loadCaptureStatus();
  unlisteners = await Promise.all([
    listen<string>("system-capture-drift", (event) => {
      if (event.payload) {
        systemMessage.value = translateMessageCode(t, event.payload);
      }
      void loadCaptureStatus();
    }),
  ]);
});

onUnmounted(() => {
  unlisteners.forEach((unlisten) => unlisten());
});
</script>

<template>
  <div class="flex min-h-full flex-col">
    <header class="border-b border-border px-6 py-4">
      <h1 class="text-lg font-semibold">{{ t("settings.title") }}</h1>
    </header>

    <main class="mx-auto w-full max-w-2xl flex-1 space-y-8 p-6">
      <PendingCaptureBanner />

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">
          {{ t("settings.language") }}
        </h2>
        <div class="rounded-xl border border-border bg-surface-raised p-4">
          <label class="block">
            <span class="mb-1 block text-sm">{{ t("settings.language") }}</span>
            <select
              :value="settings.locale"
              class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
              @change="updateLocale(($event.target as HTMLSelectElement).value as AppLocale)"
            >
              <option
                v-for="option in languageOptions"
                :key="option.value"
                :value="option.value"
              >
                {{ option.label }}
              </option>
            </select>
          </label>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">
          {{ t("settings.sections.capture") }}
        </h2>
        <div class="space-y-4 rounded-xl border border-border bg-surface-raised p-4">
          <label class="block">
            <span class="mb-1 block text-sm">{{ t("settings.saveFolder") }}</span>
            <input
              :value="settings.save_directory"
              type="text"
              class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
              @change="updateField('save_directory', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <label class="flex items-center gap-3">
            <input
              :checked="settings.auto_copy"
              type="checkbox"
              class="size-4 rounded border-border"
              @change="updateField('auto_copy', ($event.target as HTMLInputElement).checked)"
            />
            <span class="text-sm">{{ t("settings.autoCopy") }}</span>
          </label>
          <label class="flex items-center gap-3">
            <input
              :checked="settings.auto_save"
              type="checkbox"
              class="size-4 rounded border-border"
              @change="updateField('auto_save', ($event.target as HTMLInputElement).checked)"
            />
            <span class="text-sm">{{ t("settings.autoSave") }}</span>
          </label>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">
          {{ t("settings.sections.integrations") }}
        </h2>
        <div class="space-y-4 rounded-xl border border-border bg-surface-raised p-4">
          <label class="flex items-center gap-3">
            <input
              :checked="settings.allow_external_control"
              type="checkbox"
              class="size-4 rounded border-border"
              @change="
                updateField(
                  'allow_external_control',
                  ($event.target as HTMLInputElement).checked,
                )
              "
            />
            <span class="text-sm">{{ t("settings.allowExternalControl") }}</span>
          </label>
          <p class="text-xs text-text-muted">
            {{
              t("settings.urlSchemeHint", {
                scheme: "betterscreenshoot://capture-area",
              })
            }}
          </p>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">
          {{ t("settings.sections.captureMode") }}
        </h2>
        <div class="space-y-4 rounded-xl border border-border bg-surface-raised p-4">
          <div
            v-if="driftDetected"
            class="rounded-lg border border-amber-500/40 bg-amber-950/40 px-3 py-3 text-sm text-amber-100"
            role="alert"
          >
            <p>{{ driftMessage }}</p>
            <button
              type="button"
              class="mt-2 rounded-lg bg-amber-600/80 px-3 py-1.5 text-xs font-medium text-white hover:bg-amber-600 disabled:opacity-50"
              :disabled="systemBusy"
              @click="repairDrift"
            >
              {{ t("settings.repairState") }}
            </button>
          </div>

          <fieldset
            class="space-y-3"
            :disabled="systemBusy || captureStatus?.platform_supported === false"
          >
            <legend class="sr-only">{{ t("settings.systemCaptureModeLegend") }}</legend>

            <label
              class="flex cursor-pointer gap-3 rounded-lg border p-3 transition"
              :class="
                currentMode === 'independent'
                  ? 'border-accent bg-accent/5'
                  : 'border-border bg-surface hover:bg-border/40'
              "
            >
              <input
                type="radio"
                name="system-capture-mode"
                value="independent"
                class="mt-0.5"
                :checked="currentMode === 'independent'"
                @change="onModeChange('independent')"
              />
              <span class="space-y-1">
                <span class="block text-sm font-medium">
                  {{ t("settings.independentModeTitle") }}
                </span>
                <span class="block text-xs text-text-muted">
                  {{ independentHotkeyPreview }}
                </span>
                <span class="block text-xs text-text-muted">
                  {{
                    t("settings.independentModeHint", {
                      cmd3: "⌘⇧3",
                      cmd4: "⌘⇧4",
                      cmd5: "⌘⇧5",
                    })
                  }}
                </span>
              </span>
            </label>

            <label
              class="flex cursor-pointer gap-3 rounded-lg border p-3 transition"
              :class="
                currentMode === 'replace_system'
                  ? 'border-accent bg-accent/5'
                  : 'border-border bg-surface hover:bg-border/40'
              "
            >
              <input
                type="radio"
                name="system-capture-mode"
                value="replace_system"
                class="mt-0.5"
                :checked="currentMode === 'replace_system'"
                @change="onModeChange('replace_system')"
              />
              <span class="space-y-1">
                <span class="block text-sm font-medium">
                  {{ t("settings.replaceModeTitle") }}
                </span>
                <span class="block text-xs text-text-muted">
                  {{ replacementHotkeyPreview }}
                </span>
                <span class="block text-xs text-text-muted">
                  {{ t("settings.replaceModeHint") }}
                </span>
              </span>
            </label>
          </fieldset>

          <ul
            v-if="captureStatus?.platform_supported && captureStatus.system_shortcuts.length > 0"
            class="space-y-2 rounded-lg border border-border bg-surface px-3 py-3 text-sm"
          >
            <li
              v-for="shortcut in captureStatus.system_shortcuts"
              :key="shortcut.id"
              class="flex items-center justify-between gap-3"
            >
              <span class="text-text-muted">
                {{ shortcutLabel(shortcut.id, shortcut.label) }}
              </span>
              <span
                class="rounded-md px-2 py-0.5 text-xs"
                :class="
                  shortcut.enabled
                    ? 'bg-amber-950/50 text-amber-100'
                    : 'bg-emerald-950/50 text-emerald-100'
                "
              >
                {{
                  shortcut.enabled
                    ? t("settings.activeOnMacos")
                    : t("common.disabled")
                }}
              </span>
            </li>
          </ul>

          <button
            v-if="isReplaceMode"
            type="button"
            class="rounded-lg border border-border bg-surface px-3 py-2 text-sm hover:bg-border disabled:opacity-50"
            :disabled="systemBusy"
            @click="restoreSystemCaptures"
          >
            {{ t("settings.restoreSystemCaptures") }}
          </button>
          <p v-if="isReplaceMode" class="text-xs text-text-muted">
            {{
              t("settings.restoreSystemCapturesHint", {
                cmd3: "⌘⇧3",
                cmd4: "⌘⇧4",
                cmd5: "⌘⇧5",
              })
            }}
          </p>

          <p v-if="systemSuccess" class="text-xs text-emerald-400" role="status">
            {{ systemSuccess }}
          </p>
          <p v-if="systemMessage" class="text-xs text-red-400" role="alert">
            {{ systemMessage }}
          </p>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">
          {{ t("settings.sections.globalShortcuts") }}
        </h2>
        <div class="space-y-3 rounded-xl border border-border bg-surface-raised p-4">
          <label v-for="field in hotkeyFields" :key="field.key" class="block">
            <span class="mb-1 block text-sm">
              {{ field.label }}
              <span v-if="field.hint" class="text-text-muted"> — {{ field.hint }}</span>
            </span>
            <input
              :value="settings.hotkeys[field.key]"
              type="text"
              class="w-full rounded-lg border border-border bg-surface px-3 py-2 font-mono text-sm disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="isCaptureHotkeyLocked(field.key)"
              @change="updateHotkey(field.key, ($event.target as HTMLInputElement).value)"
            />
            <p
              v-if="isCaptureHotkeyLocked(field.key)"
              class="mt-1 text-xs text-text-muted"
            >
              {{
                t("settings.hotkeys.managedByReplace", {
                  shortcut: managedShortcut(field.key),
                })
              }}
            </p>
          </label>
        </div>
      </section>

      <AppUpdateSection />

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">
          {{ t("settings.sections.setup") }}
        </h2>
        <div class="rounded-xl border border-border bg-surface-raised p-4">
          <button
            type="button"
            class="rounded-lg border border-border bg-surface px-3 py-2 text-sm hover:bg-border/40 disabled:cursor-not-allowed disabled:opacity-50"
            :disabled="isOnOnboardingRoute"
            @click="runSetupWizardAgain"
          >
            {{ t("settings.runSetupWizard") }}
          </button>
        </div>
      </section>
    </main>

    <SystemScreenshotPermissionDialog
      :open="showReplaceDialog"
      :busy="systemBusy"
      :shortcuts="captureStatus?.system_shortcuts ?? []"
      @close="showReplaceDialog = false"
      @confirm="confirmReplaceMode"
    />
  </div>
</template>
