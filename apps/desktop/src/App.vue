<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "./stores/settings";
import {
  initLocaleFromSettings,
  normalizeLocale,
  setupLocaleListener,
} from "./i18n";
import { translateAppError, translateMessageCode } from "./i18n/resolveError";
import type { AppErrorPayload } from "./i18n/resolveError";

const router = useRouter();
const settingsStore = useSettingsStore();
const { t } = useI18n();
const isOverlay = ref(false);
const notice = ref<string | null>(null);
let unlisteners: UnlistenFn[] = [];

function showNotice(payload: string | AppErrorPayload) {
  notice.value = translateAppError(t, payload);
  window.setTimeout(() => {
    notice.value = null;
  }, 6000);
}

function showWarning(payload: string | AppErrorPayload) {
  notice.value = translateAppError(t, payload);
  window.setTimeout(() => {
    notice.value = null;
  }, 4000);
}

onMounted(async () => {
  try {
    const win = getCurrentWindow();
    isOverlay.value = win.label === "overlay";

    await settingsStore.load();
    initLocaleFromSettings(normalizeLocale(settingsStore.settings.locale));
  } catch (error) {
    notice.value =
      error instanceof Error
        ? translateAppError(t, error.message)
        : t("errors.appStartFailed");
  }

  unlisteners = await Promise.all([
    listen<string>("navigate", (event) => {
      if (event.payload) {
        router.push(event.payload);
      }
    }),
    listen<string | AppErrorPayload>("capture-error", (event) => {
      showNotice(event.payload);
    }),
    listen<string | AppErrorPayload>("capture-warning", (event) => {
      showWarning(event.payload);
    }),
    listen("editor-opened", () => {
      notice.value = null;
    }),
    listen<string>("system-capture-drift", (event) => {
      if (event.payload) {
        notice.value = translateMessageCode(t, event.payload);
      }
    }),
    setupLocaleListener(),
  ]);
});

onUnmounted(() => {
  unlisteners.forEach((unlisten) => unlisten());
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
