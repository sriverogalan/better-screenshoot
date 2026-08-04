<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { HotkeyConfig } from "@better-screenshoot/shared-types";
import { useSettingsStore } from "../../stores/settings";
import SettingsGroup from "../ui/SettingsGroup.vue";
import SettingsRow from "../ui/SettingsRow.vue";
import AppHotkeyRecorder from "../ui/AppHotkeyRecorder.vue";

const { t } = useI18n();
const settingsStore = useSettingsStore();

const props = defineProps<{
  isReplaceMode: boolean;
}>();

const settings = computed(() => settingsStore.settings);

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

function isCaptureHotkeyLocked(key: keyof HotkeyConfig) {
  return props.isReplaceMode && captureHotkeyKeys.includes(key);
}

function managedShortcut(key: keyof HotkeyConfig) {
  if (key === "capture_screen") return "⌘⇧3";
  if (key === "capture_area") return "⌘⇧4";
  return "⌘⇧5";
}

async function updateHotkey(key: keyof HotkeyConfig, value: string) {
  await settingsStore.save({
    ...settings.value,
    hotkeys: { ...settings.value.hotkeys, [key]: value },
  });
}
</script>

<template>
  <SettingsGroup :label="t('settings.sections.globalShortcuts')">
    <SettingsRow
      v-for="field in hotkeyFields"
      :key="field.key"
      layout="block"
    >
      <label class="block space-y-1.5">
        <span class="block text-sm">
          {{ field.label }}
          <span v-if="field.hint" class="text-fg-muted"> — {{ field.hint }}</span>
        </span>
        <AppHotkeyRecorder
          :model-value="settings.hotkeys[field.key]"
          :disabled="isCaptureHotkeyLocked(field.key)"
          :locked-label="
            isCaptureHotkeyLocked(field.key)
              ? managedShortcut(field.key)
              : undefined
          "
          @update:model-value="updateHotkey(field.key, $event)"
        />
        <p
          v-if="isCaptureHotkeyLocked(field.key)"
          class="text-xs text-fg-muted"
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
</template>
