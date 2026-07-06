<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { IconHistory, IconSettings } from "@tabler/icons-vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emit } from "@tauri-apps/api/event";

const { t } = useI18n();

async function navigateTo(route: string) {
  const mainWindow = WebviewWindow.getByLabel("main");
  if (mainWindow) {
    await mainWindow.show();
    await mainWindow.setFocus();
  }
  await emit("navigate", route);
  await getCurrentWindow().hide();
}
</script>

<template>
  <div class="border-t border-sep">
    <button
      type="button"
      class="flex w-full items-center justify-between px-3 py-2.5 hover:bg-elev transition-colors"
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
      class="flex w-full items-center justify-between border-t border-sep px-3 py-2.5 hover:bg-elev transition-colors"
      @click="navigateTo('/settings')"
    >
      <span class="flex items-center gap-2 text-fg">
        <IconSettings class="size-4 shrink-0" />
        <span class="text-sm">{{ t("menubar.settings") }}</span>
      </span>
      <kbd class="text-xs text-fg-muted">⌘,</kbd>
    </button>
  </div>
</template>
