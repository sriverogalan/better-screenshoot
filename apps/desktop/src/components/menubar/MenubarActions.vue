<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { IconCrop, IconDeviceDesktop } from "@tabler/icons-vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "../../stores/settings";
import { formatHotkey } from "../../lib/format-hotkey";

const { t } = useI18n();
const settingsStore = useSettingsStore();

const actions = computed(() => [
  {
    action: "capture-area" as const,
    label: t("history.captureRegion"),
    hotkey: formatHotkey(settingsStore.settings.hotkeys.capture_area),
    icon: IconCrop,
  },
  {
    action: "capture-screen" as const,
    label: t("history.captureScreen"),
    hotkey: formatHotkey(settingsStore.settings.hotkeys.capture_screen),
    icon: IconDeviceDesktop,
  },
]);

async function triggerCapture(action: "capture-area" | "capture-screen") {
  await getCurrentWindow().hide();
  await invoke("handle_capture_action", { action });
}
</script>

<template>
  <div class="grid grid-cols-2 gap-2 p-2.5">
    <button
      v-for="item in actions"
      :key="item.action"
      type="button"
      class="flex flex-col items-start gap-2 rounded-xl bg-tile p-2.5 text-left shadow-[inset_0_0_0_.5px_var(--color-sep)] outline-none transition-[background-color,transform] duration-100 hover:bg-control-hover focus-visible:shadow-[0_0_0_2px_var(--color-accent)] active:scale-[.97]"
      @click="triggerCapture(item.action)"
    >
      <span class="flex size-8 items-center justify-center rounded-full bg-accent text-white shadow-control">
        <component :is="item.icon" class="size-[18px]" stroke-width="2" />
      </span>
      <span class="min-w-0">
        <span class="block truncate text-body font-semibold text-fg">{{ item.label }}</span>
        <kbd class="block font-sans text-caption tracking-wider text-fg-muted">{{ item.hotkey }}</kbd>
      </span>
    </button>
  </div>
</template>
