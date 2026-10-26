<script setup lang="ts">
import { useI18n } from "vue-i18n";
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

const itemClass =
  "group flex h-6 w-full items-center justify-between rounded-[5px] px-2 text-body text-fg outline-none hover:bg-accent hover:text-white focus-visible:bg-accent focus-visible:text-white";
const shortcutClass =
  "font-sans text-body tracking-wider text-fg-muted group-hover:text-white/80 group-focus-visible:text-white/80";
</script>

<template>
  <div class="flex flex-col px-[5px] pb-[5px]">
    <div class="mx-2 mb-[5px] h-px bg-sep" role="separator" />

    <button type="button" :class="itemClass" @click="navigateTo('/history')">
      <span>{{ t("menubar.history") }}</span>
      <kbd :class="shortcutClass">⌘⇧H</kbd>
    </button>

    <button type="button" :class="itemClass" @click="navigateTo('/settings')">
      <span>{{ t("menubar.settings") }}</span>
      <kbd :class="shortcutClass">⌘,</kbd>
    </button>

    <div class="mx-2 my-[5px] h-px bg-sep" role="separator" />

    <button type="button" :class="itemClass" @click="quitApp">
      <span>{{ t("menubar.quit") }}</span>
    </button>
  </div>
</template>
