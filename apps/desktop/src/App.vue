<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  setMainWindowGuardPaused,
  startMainWindowGuard,
} from "./lib/main-window-guard";
import { useSettingsStore } from "./stores/settings";

const router = useRouter();
const settingsStore = useSettingsStore();
const isOverlay = ref(false);
const notice = ref<string | null>(null);
let stopMainWindowGuard: (() => void) | undefined;

onMounted(async () => {
  try {
    const win = getCurrentWindow();
    isOverlay.value = win.label === "overlay";

    if (win.label === "main") {
      stopMainWindowGuard = startMainWindowGuard(win);
    }

    await settingsStore.load();
  } catch (error) {
    notice.value =
      error instanceof Error ? error.message : "Error starting application";
  }

  await listen<string>("navigate", (event) => {
    if (event.payload) {
      router.push(event.payload);
    }
  });

  await listen<string>("capture-error", (event) => {
    notice.value = event.payload;
    window.setTimeout(() => {
      notice.value = null;
    }, 6000);
  });

  await listen<string>("capture-warning", (event) => {
    notice.value = event.payload;
    window.setTimeout(() => {
      notice.value = null;
    }, 4000);
  });

  await listen("capture-session-active", () => {
    setMainWindowGuardPaused(true);
  });

  await listen("editor-opened", () => {
    setMainWindowGuardPaused(true);
    notice.value = null;
  });

  await listen("editor-closed", () => {
    setMainWindowGuardPaused(false);
  });
});

onUnmounted(() => {
  stopMainWindowGuard?.();
});
</script>

<template>
  <div
    :class="
      isOverlay
        ? 'min-h-full bg-transparent'
        : 'h-full bg-[#111318] text-[#e8eaed]'
    "
  >
    <div
      v-if="notice && !isOverlay"
      class="fixed inset-x-4 top-4 z-50 rounded-xl border border-red-500/40 bg-red-950/90 px-4 py-3 text-sm text-red-100 shadow-lg"
      role="alert"
    >
      {{ notice }}
    </div>
    <router-view class="h-full min-h-0" />
  </div>
</template>
