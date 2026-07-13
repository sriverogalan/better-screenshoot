<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  DEFAULT_HOTKEYS,
  SYSTEM_REPLACEMENT_HOTKEYS,
  type AppAppearance,
  type AppLocale,
  type HotkeyConfig,
  type SystemCaptureMode,
} from "@better-screenshoot/shared-types";
import { useSettingsStore } from "../stores/settings";
import { useAppearance } from "../composables/useAppearance";
import AppToggle from "../components/ui/AppToggle.vue";
import AppSegmentedControl from "../components/ui/AppSegmentedControl.vue";
import AppButton from "../components/ui/AppButton.vue";
import AlertBanner from "../components/ui/AlertBanner.vue";
import SettingsGroup from "../components/ui/SettingsGroup.vue";
import SettingsRow from "../components/ui/SettingsRow.vue";
import SystemScreenshotPermissionDialog from "../components/settings/SystemScreenshotPermissionDialog.vue";
import AppUpdateSection from "../components/settings/AppUpdateSection.vue";
import {
  getCaptureStatus,
  getSystemCaptureStatus,
  openScreenRecordingSettings,
  resetScreenCapturePermission,
  setSystemCaptureMode,
  type CaptureStatus,
  type SystemCaptureStatus,
} from "../lib/tauri";
import { formatHotkey } from "../lib/format-hotkey";
import PendingCaptureBanner from "../components/PendingCaptureBanner.vue";
import { SUPPORTED_LOCALES, setLocale } from "../i18n";
import { translateAppError, translateMessageCode } from "../i18n/resolveError";
import { systemShortcutLabelKey } from "../lib/system-shortcut-labels";
import { deriveCapturePermissionPresentation } from "../lib/permission-presentation";

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const settingsStore = useSettingsStore();
const { applyAppearance } = useAppearance();

const hotkeyFields = computed(() => [
  {
    key: "capture_area" as const,
    label: t("settings.hotkeys.captureArea"),
    hint: t("settings.hotkeys.captureAreaHint"),
  },
  { key: "capture_screen" as const, label: t("settings.hotkeys.captureScreen") },
  { key: "open_history" as const, label: t("settings.hotkeys.openHistory") },
]);

const captureHotkeyKeys: Array<keyof HotkeyConfig> = ["capture_area", "capture_screen"];

const settings = computed(() => settingsStore.settings);
const systemMessage = ref<string | null>(null);
const systemSuccess = ref<string | null>(null);
const systemBusy = ref(false);
const showReplaceDialog = ref(false);
const captureStatus = ref<SystemCaptureStatus | null>(null);
const capturePermissionStatus = ref<CaptureStatus | null>(null);
const permissionRepairBusy = ref(false);

const languageOptions = computed(() =>
  SUPPORTED_LOCALES.map((locale) => ({
    value: locale,
    label: t(`settings.languageOptions.${locale}`),
  })),
);

const appearanceOptions = computed(() => [
  { value: "auto", label: t("settings.appearanceOptions.auto") },
  { value: "light", label: t("settings.appearanceOptions.light") },
  { value: "dark", label: t("settings.appearanceOptions.dark") },
]);

const independentHotkeyPreview = computed(() =>
  [
    `${formatHotkey(DEFAULT_HOTKEYS.capture_area)} ${t("common.region")}`,
    `${formatHotkey(DEFAULT_HOTKEYS.capture_screen)} ${t("common.screen")}`,
  ].join(" · "),
);

const replacementHotkeyPreview = computed(() =>
  [
    `${formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_screen)} ${t("common.screen")}`,
    `${formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_area)} ${t("common.region")}`,
  ].join(" · "),
);

const currentMode = computed(() => settings.value.system_capture_mode);
const isReplaceMode = computed(() => currentMode.value === "replace_system");
const driftDetected = computed(() => captureStatus.value?.drift_detected ?? false);
const driftMessage = computed(() => {
  const code = captureStatus.value?.messageCode;
  return code ? translateMessageCode(t, code) : null;
});
const capturePermissionPresentation = computed(() =>
  capturePermissionStatus.value
    ? deriveCapturePermissionPresentation(capturePermissionStatus.value)
    : null,
);

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

async function onAppearanceChange(value: string) {
  const appearance = value as AppAppearance;
  await settingsStore.save({ ...settings.value, appearance });
  applyAppearance(appearance);
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

async function loadCapturePermissionStatus() {
  try {
    capturePermissionStatus.value = await getCaptureStatus();
  } catch (err) {
    systemMessage.value =
      err instanceof Error
        ? translateAppError(t, err.message)
        : t("errors.checkPermissionsFailed");
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

async function repairScreenRecordingPermission() {
  if (permissionRepairBusy.value) return;

  const confirmed = window.confirm(t("settings.repairScreenRecordingConfirm"));
  if (!confirmed) return;

  permissionRepairBusy.value = true;
  systemMessage.value = null;
  systemSuccess.value = null;
  try {
    await resetScreenCapturePermission();
    try {
      await openScreenRecordingSettings();
    } catch {
      systemMessage.value = t("errors.openSystemSettingsFailed");
    }
    systemSuccess.value = t("settings.repairScreenRecordingStarted");
    await loadCapturePermissionStatus();
    await loadCaptureStatus();
  } catch (err) {
    systemMessage.value =
      err instanceof Error
        ? translateAppError(t, err.message)
        : t("errors.repairScreenRecordingPermissionFailed");
  } finally {
    permissionRepairBusy.value = false;
  }
}

let unlisteners: UnlistenFn[] = [];

onMounted(async () => {
  await Promise.all([loadCaptureStatus(), loadCapturePermissionStatus()]);
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
  <div class="flex min-h-full flex-col bg-win">
    <div data-tauri-drag-region class="h-8 shrink-0" />

    <h1 class="px-6 pb-2 pt-4 text-2xl font-bold text-fg">
      {{ t("settings.title") }}
    </h1>

    <main class="flex-1 overflow-y-auto pb-8">
      <div class="mx-auto w-full max-w-2xl px-6">
        <PendingCaptureBanner />

        <!-- Idioma -->
        <SettingsGroup :label="t('settings.language')">
          <SettingsRow>
            <span class="text-sm">{{ t("settings.language") }}</span>
            <select
              :value="settings.locale"
              class="rounded-lg border border-sep bg-field px-3 py-1.5 text-sm text-fg"
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
          </SettingsRow>
        </SettingsGroup>

        <!-- Captura -->
        <SettingsGroup :label="t('settings.sections.capture')">
          <SettingsRow>
            <span class="min-w-0 flex-1 truncate text-sm text-fg-muted">
              {{ settings.save_directory || t("settings.saveFolder") }}
            </span>
            <input
              :value="settings.save_directory"
              type="text"
              class="ml-4 w-48 shrink-0 rounded-lg border border-sep bg-field px-3 py-1.5 text-sm text-fg"
              :placeholder="t('settings.saveFolder')"
              @change="updateField('save_directory', ($event.target as HTMLInputElement).value)"
            />
          </SettingsRow>
          <SettingsRow>
            <span class="text-sm">{{ t("settings.autoCopy") }}</span>
            <AppToggle
              :model-value="settings.auto_copy"
              @update:model-value="updateField('auto_copy', $event)"
            />
          </SettingsRow>
          <SettingsRow>
            <span class="text-sm">{{ t("settings.autoSave") }}</span>
            <AppToggle
              :model-value="settings.auto_save"
              @update:model-value="updateField('auto_save', $event)"
            />
          </SettingsRow>
        </SettingsGroup>

        <!-- Aspecto -->
        <SettingsGroup :label="t('settings.sections.appearance')">
          <SettingsRow>
            <span class="text-sm">{{ t("settings.sections.appearance") }}</span>
            <AppSegmentedControl
              :model-value="settings.appearance"
              :options="appearanceOptions"
              @update:model-value="onAppearanceChange"
            />
          </SettingsRow>
        </SettingsGroup>

        <!-- Integraciones -->
        <SettingsGroup :label="t('settings.sections.integrations')">
          <SettingsRow>
            <span class="text-sm">{{ t("settings.allowExternalControl") }}</span>
            <AppToggle
              :model-value="settings.allow_external_control"
              @update:model-value="updateField('allow_external_control', $event)"
            />
          </SettingsRow>
          <SettingsRow layout="block">
            <p class="text-xs text-fg-muted">
              {{
                t("settings.urlSchemeHint", {
                  scheme: "betterscreenshoot://capture-area",
                })
              }}
            </p>
          </SettingsRow>
        </SettingsGroup>

        <!-- Modo de captura -->
        <SettingsGroup :label="t('settings.sections.captureMode')">
          <SettingsRow layout="block" class="space-y-4">
            <AlertBanner
              v-if="capturePermissionPresentation?.showRepairAction"
              tone="warning"
            >
              <p>{{ t("settings.repairScreenRecordingDescription") }}</p>
              <p class="text-xs text-amber-100/80">
                {{ t(capturePermissionPresentation.impactCode) }}
              </p>
              <template #actions>
                <AppButton
                  variant="secondary"
                  :disabled="permissionRepairBusy"
                  @click="repairScreenRecordingPermission"
                >
                  {{ t("settings.repairScreenRecordingPermission") }}
                </AppButton>
              </template>
            </AlertBanner>

            <AlertBanner v-if="driftDetected" tone="warning">
              <p>{{ driftMessage }}</p>
              <template #actions>
                <AppButton variant="secondary" :disabled="systemBusy" @click="repairDrift">
                  {{ t("settings.repairState") }}
                </AppButton>
              </template>
            </AlertBanner>

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
                    : 'border-sep bg-elev hover:bg-elev/80'
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
                  <span class="block text-xs text-fg-muted">
                    {{ independentHotkeyPreview }}
                  </span>
                  <span class="block text-xs text-fg-muted">
                    {{
                      t("settings.independentModeHint", {
                        cmd3: "⌘⇧3",
                        cmd4: "⌘⇧4",
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
                    : 'border-sep bg-elev hover:bg-elev/80'
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
                  <span class="block text-xs text-fg-muted">
                    {{ replacementHotkeyPreview }}
                  </span>
                  <span class="block text-xs text-fg-muted">
                    {{ t("settings.replaceModeHint") }}
                  </span>
                </span>
              </label>
            </fieldset>

            <ul
              v-if="captureStatus?.platform_supported && captureStatus.system_shortcuts.length > 0"
              class="mt-3 space-y-2 rounded-lg border border-sep bg-elev px-3 py-3 text-sm"
            >
              <li
                v-for="shortcut in captureStatus.system_shortcuts"
                :key="shortcut.id"
                class="flex items-center justify-between gap-3"
              >
                <span class="text-fg-muted">
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

            <div v-if="isReplaceMode">
              <AppButton variant="secondary" :disabled="systemBusy" @click="restoreSystemCaptures">
                {{ t("settings.restoreSystemCaptures") }}
              </AppButton>
              <p class="mt-2 text-xs text-fg-muted">
                {{
                  t("settings.restoreSystemCapturesHint", {
                    cmd3: "⌘⇧3",
                    cmd4: "⌘⇧4",
                  })
                }}
              </p>
            </div>

            <p v-if="systemSuccess" class="text-xs text-success" role="status">
              {{ systemSuccess }}
            </p>
            <p v-if="systemMessage" class="text-xs text-danger" role="alert">
              {{ systemMessage }}
            </p>
          </SettingsRow>
        </SettingsGroup>

        <!-- Atajos globales -->
        <SettingsGroup :label="t('settings.sections.globalShortcuts')">
          <SettingsRow layout="block" class="space-y-3">
            <label v-for="field in hotkeyFields" :key="field.key" class="block">
              <span class="mb-1 block text-sm">
                {{ field.label }}
                <span v-if="field.hint" class="text-fg-muted"> — {{ field.hint }}</span>
              </span>
              <input
                :value="settings.hotkeys[field.key]"
                type="text"
                class="w-full rounded-lg border border-sep bg-field px-3 py-2 font-mono text-sm text-fg disabled:cursor-not-allowed disabled:opacity-60"
                :disabled="isCaptureHotkeyLocked(field.key)"
                @change="updateHotkey(field.key, ($event.target as HTMLInputElement).value)"
              />
              <p
                v-if="isCaptureHotkeyLocked(field.key)"
                class="mt-1 text-xs text-fg-muted"
              >
                {{
                  t("settings.hotkeys.managedByReplace", {
                    shortcut: managedShortcut(field.key),
                  })
                }}
              </p>
            </label>
          </SettingsRow>
        </SettingsGroup>

        <!-- Actualizaciones -->
        <div class="mt-6">
          <AppUpdateSection />
        </div>

        <!-- Configuración -->
        <SettingsGroup :label="t('settings.sections.setup')">
          <SettingsRow>
            <AppButton
              variant="secondary"
              :disabled="isOnOnboardingRoute"
              @click="runSetupWizardAgain"
            >
              {{ t("settings.runSetupWizard") }}
            </AppButton>
          </SettingsRow>
        </SettingsGroup>
      </div>
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
