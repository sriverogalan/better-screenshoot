<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { IconCrop, IconPhoto } from "@tabler/icons-vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../../stores/settings";
import { formatHotkey } from "../../lib/format-hotkey";

const { t } = useI18n();
const settingsStore = useSettingsStore();

async function triggerCapture(action: "capture-area" | "capture-screen") {
  await getCurrentWindow().hide();
  await invoke("handle_capture_action", { action });
}
</script>

<template>
  <div class="flex flex-col gap-0.5 p-1.5">
    <button
      type="button"
      class="flex items-center justify-between rounded-lg px-2.5 py-2 bg-accent text-white transition-colors active:bg-accent/80"
      @click="triggerCapture('capture-area')"
    >
      <span class="flex items-center gap-2">
        <IconCrop class="size-4 shrink-0" />
        <span class="text-sm font-medium">{{ t("history.captureRegion") }}</span>
      </span>
      <kbd class="text-xs opacity-70">
        {{ formatHotkey(settingsStore.settings.hotkeys.capture_area) }}
      </kbd>
    </button>

    <button
      type="button"
      class="flex items-center justify-between rounded-lg px-2.5 py-2 hover:bg-elev active:bg-elev/70 transition-colors"
      @click="triggerCapture('capture-screen')"
    >
      <span class="flex items-center gap-2 text-fg">
        <IconPhoto class="size-4 shrink-0" />
        <span class="text-sm">{{ t("history.captureScreen") }}</span>
      </span>
      <kbd class="text-xs text-fg-muted">
        {{ formatHotkey(settingsStore.settings.hotkeys.capture_screen) }}
      </kbd>
    </button>
  </div>
</template>
