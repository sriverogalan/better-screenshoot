<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { IconHistory, IconSettings, IconPower } from "@tabler/icons-vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emitTo } from "@tauri-apps/api/event";
import { exit } from "@tauri-apps/plugin-process";

const { t } = useI18n();

async function navigateTo(route: string) {
  const mainWindow = await WebviewWindow.getByLabel("main");
  if (mainWindow) {
    await mainWindow.show();
    await mainWindow.setFocus();
  }
  await emitTo("main", "navigate", route);
  await getCurrentWindow().hide();
}

async function quitApp() {
  await exit(0);
}
</script>

<template>
  <div class="flex flex-col gap-0.5 border-t border-sep p-1.5">
    <button
      type="button"
      class="flex w-full items-center justify-between rounded-lg px-2.5 py-2 hover:bg-elev active:bg-elev/70 transition-colors"
      @click="navigateTo('/history')"
    >
      <span class="flex items-center gap-2 text-fg">
        <IconHistory class="size-4 shrink-0" />
        <span class="text-sm">{{ t("menubar.history") }}</span>
      </span>
      <kbd class="text-xs text-fg-muted">⌘⇧H</kbd>
    </button>

    <button
      type="button"
      class="flex w-full items-center justify-between rounded-lg px-2.5 py-2 hover:bg-elev active:bg-elev/70 transition-colors"
      @click="navigateTo('/settings')"
    >
      <span class="flex items-center gap-2 text-fg">
        <IconSettings class="size-4 shrink-0" />
        <span class="text-sm">{{ t("menubar.settings") }}</span>
      </span>
      <kbd class="text-xs text-fg-muted">⌘,</kbd>
    </button>

    <div class="my-1 border-t border-sep"></div>

    <button
      type="button"
      class="flex w-full items-center justify-between rounded-lg px-2.5 py-2 hover:bg-elev active:bg-elev/70 transition-colors"
      @click="quitApp"
    >
      <span class="flex items-center gap-2 text-fg">
        <IconPower class="size-4 shrink-0" />
        <span class="text-sm">{{ t("menubar.quit") }}</span>
      </span>
    </button>
  </div>
</template>
