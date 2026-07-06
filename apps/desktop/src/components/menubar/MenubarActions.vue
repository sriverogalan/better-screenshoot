<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { IconCrop, IconPhoto, IconAppWindow } from "@tabler/icons-vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../../stores/settings";
import { formatHotkey } from "../../lib/format-hotkey";

const { t } = useI18n();
const settingsStore = useSettingsStore();

async function triggerCapture(action: "capture-area" | "capture-screen" | "capture-window") {
  await getCurrentWindow().hide();
  await invoke("handle_capture_action", { action });
}
</script>

<template>
  <div class="flex flex-col">
    <button
      type="button"
      class="flex items-center justify-between px-3 py-2.5 bg-accent text-white hover:bg-accent/90 transition-colors"
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
      class="flex items-center justify-between border-b border-sep px-3 py-2.5 hover:bg-elev transition-colors"
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

    <button
      type="button"
      class="flex items-center justify-between border-b border-sep px-3 py-2.5 hover:bg-elev transition-colors"
      @click="triggerCapture('capture-window')"
    >
      <span class="flex items-center gap-2 text-fg">
        <IconAppWindow class="size-4 shrink-0" />
        <span class="text-sm">{{ t("history.captureWindow") }}</span>
      </span>
      <kbd class="text-xs text-fg-muted">
        {{ formatHotkey(settingsStore.settings.hotkeys.capture_window) }}
      </kbd>
    </button>
  </div>
</template>
