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
  <div v-if="capturePermissionPresentation?.showRepairAction || driftDetected" class="mb-6 space-y-3">
    <AlertBanner v-if="capturePermissionPresentation?.showRepairAction" tone="warning">
      <p class="font-medium">{{ t("settings.repairScreenRecordingDescription") }}</p>
      <p class="text-subhead text-fg-muted">
        {{ t(capturePermissionPresentation.impactCode) }}
      </p>
      <template #actions>
        <AppButton
          size="sm"
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
        <AppButton size="sm" :disabled="systemBusy" @click="repairDrift">
          {{ t("settings.repairState") }}
        </AppButton>
      </template>
    </AlertBanner>
  </div>

  <SettingsGroup v-if="capturePermissionPresentation">
    <SettingsRow
      :label="t('onboarding.stepPermission.title')"
      :description="t(capturePermissionPresentation.impactCode)"
    >
      <AppBadge :tone="capturePermissionPresentation.status === 'granted' ? 'success' : 'warning'">
        {{
          capturePermissionPresentation.status === "granted"
            ? t("settings.permissionGranted")
            : t("settings.permissionMissing")
        }}
      </AppBadge>
    </SettingsRow>
  </SettingsGroup>

  <SettingsGroup :label="t('settings.systemCaptureModeLegend')">
    <fieldset
      class="contents"
      :disabled="systemBusy || captureStatus?.platform_supported === false"
    >
      <legend class="sr-only">{{ t("settings.systemCaptureModeLegend") }}</legend>
      <button
        v-for="option in modeOptions"
        :key="option.value"
        type="button"
        role="radio"
        class="relative flex w-full gap-3 px-3 py-3 text-left transition-colors before:absolute before:inset-x-3 before:top-0 before:h-px before:bg-sep first-of-type:before:hidden enabled:hover:bg-hover disabled:opacity-50"
        :aria-checked="currentMode === option.value"
        :disabled="systemBusy || captureStatus?.platform_supported === false"
        @click="onModeChange(option.value)"
      >
        <span
          class="mt-px flex size-4 shrink-0 items-center justify-center rounded-full transition-colors"
          :class="
            currentMode === option.value
              ? 'bg-accent shadow-control'
              : 'bg-field shadow-[inset_0_0_0_1px_var(--color-fg-tertiary)]'
          "
          aria-hidden="true"
        >
          <span v-if="currentMode === option.value" class="size-1.5 rounded-full bg-white" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="block text-body font-medium text-fg">{{ option.title }}</span>
          <span class="mt-0.5 block text-subhead text-fg-muted">{{ option.hint }}</span>
          <span class="mt-1.5 block text-subhead font-medium tracking-wide text-fg-muted">
            {{ option.preview }}
          </span>
        </span>
      </button>
    </fieldset>
  </SettingsGroup>

  <SettingsGroup
    v-if="captureStatus?.platform_supported && captureStatus.system_shortcuts.length > 0"
    :label="t('settings.systemShortcutsTitle')"
  >
    <SettingsRow
      v-for="shortcut in captureStatus.system_shortcuts"
      :key="shortcut.id"
      :label="shortcutLabel(shortcut.id, shortcut.label)"
    >
      <AppBadge :tone="shortcut.enabled ? 'warning' : 'success'">
        {{
          shortcut.enabled
            ? t("settings.activeOnMacos")
            : t("common.disabled")
        }}
      </AppBadge>
    </SettingsRow>
  </SettingsGroup>

  <SettingsGroup v-if="isReplaceMode">
    <SettingsRow
      :label="t('settings.restoreSystemCaptures')"
      :description="t('settings.restoreSystemCapturesHint', { cmd3: '⌘⇧3', cmd4: '⌘⇧4' })"
    >
      <AppButton :disabled="systemBusy" @click="restoreSystemCaptures">
        {{ t("settings.restoreSystemCaptures") }}
      </AppButton>
    </SettingsRow>
  </SettingsGroup>

  <p v-if="systemSuccess" class="mt-3 px-2.5 text-subhead text-success" role="status">
    {{ systemSuccess }}
  </p>
  <p v-if="systemMessage" class="mt-3 px-2.5 text-subhead text-danger" role="alert">
    {{ systemMessage }}
  </p>

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
