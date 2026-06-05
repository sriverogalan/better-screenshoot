<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { TIERS } from "@better-screenshoot/licensing";
import {
  DEFAULT_HOTKEYS,
  SYSTEM_REPLACEMENT_HOTKEYS,
  type HotkeyConfig,
  type LicenseTier,
  type SystemCaptureMode,
} from "@better-screenshoot/shared-types";
import { useSettingsStore } from "../stores/settings";
import SystemScreenshotPermissionDialog from "../components/settings/SystemScreenshotPermissionDialog.vue";
import {
  getSystemCaptureStatus,
  setSystemCaptureMode,
  type SystemCaptureStatus,
  validateLicenseKey,
} from "../lib/tauri";
import { formatHotkey } from "../lib/format-hotkey";
import PendingCaptureBanner from "../components/PendingCaptureBanner.vue";

const settingsStore = useSettingsStore();

const hotkeyFields: Array<{ key: keyof HotkeyConfig; label: string; hint?: string }> = [
  { key: "capture_area", label: "Capture region", hint: "On macOS opens the native system selector" },
  { key: "capture_screen", label: "Capture screen" },
  { key: "capture_window", label: "Capture window" },
  { key: "open_history", label: "Open history" },
];

const captureHotkeyKeys: Array<keyof HotkeyConfig> = [
  "capture_area",
  "capture_screen",
  "capture_window",
];

const settings = computed(() => settingsStore.settings);
const licenseKey = ref("");
const licenseMessage = ref("");
const systemMessage = ref<string | null>(null);
const systemSuccess = ref<string | null>(null);
const systemBusy = ref(false);
const showReplaceDialog = ref(false);
const captureStatus = ref<SystemCaptureStatus | null>(null);

const independentHotkeyPreview = computed(() =>
  [
    `${formatHotkey(DEFAULT_HOTKEYS.capture_area)} region`,
    `${formatHotkey(DEFAULT_HOTKEYS.capture_screen)} screen`,
    `${formatHotkey(DEFAULT_HOTKEYS.capture_window)} window`,
  ].join(" · "),
);

const replacementHotkeyPreview = computed(() =>
  [
    `${formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_screen)} screen`,
    `${formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_area)} region`,
    `${formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_window)} window`,
  ].join(" · "),
);

const currentMode = computed(() => settings.value.system_capture_mode);
const isReplaceMode = computed(() => currentMode.value === "replace_system");
const driftDetected = computed(() => captureStatus.value?.drift_detected ?? false);
const driftMessage = computed(() => captureStatus.value?.message ?? null);

function isCaptureHotkeyLocked(key: keyof HotkeyConfig) {
  return isReplaceMode.value && captureHotkeyKeys.includes(key);
}

async function applyLicense() {
  const result = await validateLicenseKey(licenseKey.value);
  licenseMessage.value = result.message;
  if (result.valid) {
    await settingsStore.save({
      ...settings.value,
      tier: result.tier as LicenseTier,
    });
  }
}

const tierLabels: Record<LicenseTier, string> = {
  community: "Community (free)",
  pro: "Pro",
  cloud: "Cloud",
  team: "Team",
};

async function updateField<K extends keyof typeof settings.value>(
  key: K,
  value: (typeof settings.value)[K],
) {
  await settingsStore.save({ ...settings.value, [key]: value });
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
      err instanceof Error ? err.message : "Could not check capture mode";
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
    systemSuccess.value = result.message;
  } catch (err) {
    systemMessage.value =
      err instanceof Error ? err.message : "Could not change capture mode";
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
      systemMessage.value = event.payload;
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
      <h1 class="text-lg font-semibold">Settings</h1>
    </header>

    <main class="mx-auto w-full max-w-2xl flex-1 space-y-8 p-6">
      <PendingCaptureBanner />

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">Capture</h2>
        <div class="space-y-4 rounded-xl border border-border bg-surface-raised p-4">
          <label class="block">
            <span class="mb-1 block text-sm">Save folder</span>
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
            <span class="text-sm">Copy to clipboard automatically</span>
          </label>
          <label class="flex items-center gap-3">
            <input
              :checked="settings.auto_save"
              type="checkbox"
              class="size-4 rounded border-border"
              @change="updateField('auto_save', ($event.target as HTMLInputElement).checked)"
            />
            <span class="text-sm">Save captures to disk</span>
          </label>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">Integrations</h2>
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
            <span class="text-sm">Allow external control (Raycast, CLI, URL scheme)</span>
          </label>
          <p class="text-xs text-text-muted">
            URL scheme: <code class="text-accent">betterscreenshoot://capture-area</code>
          </p>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">Capture mode (macOS)</h2>
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
              Repair state
            </button>
          </div>

          <fieldset
            class="space-y-3"
            :disabled="systemBusy || captureStatus?.platform_supported === false"
          >
            <legend class="sr-only">System capture mode</legend>

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
                <span class="block text-sm font-medium">Better Screenshoot shortcuts</span>
                <span class="block text-xs text-text-muted">
                  {{ independentHotkeyPreview }}
                </span>
                <span class="block text-xs text-text-muted">
                  macOS keeps <code class="text-accent">⌘⇧3</code>,
                  <code class="text-accent">⌘⇧4</code> y
                  <code class="text-accent">⌘⇧5</code>.
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
                <span class="block text-sm font-medium">Replace system captures</span>
                <span class="block text-xs text-text-muted">
                  {{ replacementHotkeyPreview }}
                </span>
                <span class="block text-xs text-text-muted">
                  Disables native shortcuts and reassigns them to Better Screenshoot.
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
              <span class="text-text-muted">{{ shortcut.label }}</span>
              <span
                class="rounded-md px-2 py-0.5 text-xs"
                :class="
                  shortcut.enabled
                    ? 'bg-amber-950/50 text-amber-100'
                    : 'bg-emerald-950/50 text-emerald-100'
                "
              >
                {{ shortcut.enabled ? "Active on macOS" : "Disabled" }}
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
            Restore system captures
          </button>
          <p v-if="isReplaceMode" class="text-xs text-text-muted">
            Re-enables macOS <code class="text-accent">⌘⇧3</code>,
            <code class="text-accent">⌘⇧4</code>, and
            <code class="text-accent">⌘⇧5</code> and returns to the app's own shortcuts.
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
        <h2 class="mb-4 text-sm font-medium text-text-muted">Global shortcuts</h2>
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
              Managed by «Replace system captures» ({{
                field.key === "capture_screen"
                  ? "⌘⇧3"
                  : field.key === "capture_area"
                    ? "⌘⇧4"
                    : "⌘⇧5"
              }}).
            </p>
          </label>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">License</h2>
        <div class="space-y-4 rounded-xl border border-border bg-surface-raised p-4">
          <p class="text-sm">
            Current plan:
            <span class="font-medium text-accent">{{ tierLabels[settings.tier] }}</span>
          </p>
          <label class="block">
            <span class="mb-1 block text-sm">License key</span>
            <input
              v-model="licenseKey"
              type="text"
              placeholder="BS-PRO-..."
              class="w-full rounded-lg border border-border bg-surface px-3 py-2 font-mono text-sm"
            />
          </label>
          <button
            type="button"
            class="rounded-lg bg-accent px-4 py-2 text-sm text-white hover:bg-accent-hover"
            @click="applyLicense"
          >
            Activate license
          </button>
          <p v-if="licenseMessage" class="text-xs text-text-muted">{{ licenseMessage }}</p>
          <ul class="space-y-2 text-xs text-text-muted">
            <li v-for="tier in Object.values(TIERS)" :key="tier.id">
              <strong class="text-text">{{ tier.name }}</strong> — {{ tier.price }}:
              {{ tier.features.slice(0, 2).join(", ") }}…
            </li>
          </ul>
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
