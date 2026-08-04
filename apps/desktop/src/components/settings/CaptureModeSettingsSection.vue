<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  DEFAULT_HOTKEYS,
  SYSTEM_REPLACEMENT_HOTKEYS,
  type SystemCaptureMode,
} from "@better-screenshoot/shared-types";
import { useSettingsStore } from "../../stores/settings";
import {
  getCaptureStatus,
  getSystemCaptureStatus,
  openScreenRecordingSettings,
  resetScreenCapturePermission,
  setSystemCaptureMode,
  type CaptureStatus,
  type SystemCaptureStatus,
} from "../../lib/tauri";
import { formatHotkey } from "../../lib/format-hotkey";
import { translateAppError, translateMessageCode } from "../../i18n/resolveError";
import { systemShortcutLabelKey } from "../../lib/system-shortcut-labels";
import { deriveCapturePermissionPresentation } from "../../lib/permission-presentation";
import SettingsGroup from "../ui/SettingsGroup.vue";
import SettingsRow from "../ui/SettingsRow.vue";
import AlertBanner from "../ui/AlertBanner.vue";
import AppButton from "../ui/AppButton.vue";
import AppBadge from "../ui/AppBadge.vue";
import AppDialog from "../ui/AppDialog.vue";
import SystemScreenshotPermissionDialog from "./SystemScreenshotPermissionDialog.vue";

const { t } = useI18n();
const settingsStore = useSettingsStore();

const settings = computed(() => settingsStore.settings);
const systemMessage = ref<string | null>(null);
const systemSuccess = ref<string | null>(null);
const systemBusy = ref(false);
const showReplaceDialog = ref(false);
const showRepairDialog = ref(false);
const captureStatus = ref<SystemCaptureStatus | null>(null);
const capturePermissionStatus = ref<CaptureStatus | null>(null);
const permissionRepairBusy = ref(false);

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

function shortcutLabel(id: number, fallback: string) {
  const key = systemShortcutLabelKey(id);
  return key ? t(key) : fallback;
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

  permissionRepairBusy.value = true;
  systemMessage.value = null;
  systemSuccess.value = null;
  showRepairDialog.value = false;
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

const modeOptions = computed(() => [
  {
    value: "independent" as const,
    title: t("settings.independentModeTitle"),
    preview: independentHotkeyPreview.value,
    hint: t("settings.independentModeHint", { cmd3: "⌘⇧3", cmd4: "⌘⇧4" }),
  },
  {
    value: "replace_system" as const,
    title: t("settings.replaceModeTitle"),
    preview: replacementHotkeyPreview.value,
    hint: t("settings.replaceModeHint"),
  },
]);

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
  <SettingsGroup :label="t('settings.sections.captureMode')">
    <SettingsRow layout="block">
      <div class="space-y-4">
        <AlertBanner
          v-if="capturePermissionPresentation?.showRepairAction"
          tone="warning"
        >
          <p>{{ t("settings.repairScreenRecordingDescription") }}</p>
          <p class="text-xs opacity-80">
            {{ t(capturePermissionPresentation.impactCode) }}
          </p>
          <template #actions>
            <AppButton
              variant="secondary"
              :disabled="permissionRepairBusy"
              @click="showRepairDialog = true"
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

          <button
            v-for="option in modeOptions"
            :key="option.value"
            type="button"
            role="radio"
            class="flex w-full cursor-pointer gap-3 rounded-lg border p-3 text-left transition"
            :class="
              currentMode === option.value
                ? 'border-accent bg-accent/5'
                : 'border-sep bg-win hover:bg-win/80'
            "
            :aria-checked="currentMode === option.value"
            :disabled="systemBusy || captureStatus?.platform_supported === false"
            @click="onModeChange(option.value)"
          >
            <span
              class="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border"
              :class="
                currentMode === option.value
                  ? 'border-accent'
                  : 'border-fg-muted/50'
              "
              aria-hidden="true"
            >
              <span
                v-if="currentMode === option.value"
                class="size-2 rounded-full bg-accent"
              />
            </span>
            <span class="min-w-0 space-y-1">
              <span class="block text-sm font-medium">{{ option.title }}</span>
              <span class="block text-xs text-fg-muted">{{ option.preview }}</span>
              <span class="block text-xs text-fg-muted">{{ option.hint }}</span>
            </span>
          </button>
        </fieldset>

        <ul
          v-if="captureStatus?.platform_supported && captureStatus.system_shortcuts.length > 0"
          class="space-y-2 rounded-lg border border-sep bg-win px-3 py-3 text-sm"
        >
          <li
            v-for="shortcut in captureStatus.system_shortcuts"
            :key="shortcut.id"
            class="flex items-center justify-between gap-3"
          >
            <span class="text-fg-muted">
              {{ shortcutLabel(shortcut.id, shortcut.label) }}
            </span>
            <AppBadge :tone="shortcut.enabled ? 'warning' : 'success'">
              {{
                shortcut.enabled
                  ? t("settings.activeOnMacos")
                  : t("common.disabled")
              }}
            </AppBadge>
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
      </div>
    </SettingsRow>
  </SettingsGroup>

  <SystemScreenshotPermissionDialog
    :open="showReplaceDialog"
    :busy="systemBusy"
    :shortcuts="captureStatus?.system_shortcuts ?? []"
    @close="showReplaceDialog = false"
    @confirm="confirmReplaceMode"
  />

  <AppDialog
    :open="showRepairDialog"
    :title="t('settings.repairScreenRecordingPermission')"
    :description="t('settings.repairScreenRecordingConfirm')"
    :busy="permissionRepairBusy"
    @close="showRepairDialog = false"
  >
    <template #footer>
      <AppButton
        variant="secondary"
        :disabled="permissionRepairBusy"
        @click="showRepairDialog = false"
      >
        {{ t("common.cancel") }}
      </AppButton>
      <AppButton
        variant="primary"
        :disabled="permissionRepairBusy"
        @click="repairScreenRecordingPermission"
      >
        {{ t("settings.repairScreenRecordingPermission") }}
      </AppButton>
    </template>
  </AppDialog>
</template>
