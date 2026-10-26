<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useI18n } from "vue-i18n";
import { IconExclamationCircleFilled } from "@tabler/icons-vue";
import { useSettingsStore } from "./stores/settings";
import { useAppearance } from "./composables/useAppearance";
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
useAppearance();
const isOverlay = ref(false);
const notice = ref<string | null>(null);
let unlisteners: UnlistenFn[] = [];
let noticeTimeout: number | null = null;

function setNotice(payload: string | AppErrorPayload, durationMs: number) {
  if (noticeTimeout !== null) {
    window.clearTimeout(noticeTimeout);
  }
  notice.value = translateAppError(t, payload);
  noticeTimeout = window.setTimeout(() => {
    notice.value = null;
    noticeTimeout = null;
  }, durationMs);
}

function showNotice(payload: string | AppErrorPayload) {
  setNotice(payload, 3000);
}

function showWarning(payload: string | AppErrorPayload) {
  setNotice(payload, 3000);
}

onMounted(async () => {
  try {
    const win = getCurrentWindow();
    isOverlay.value = win.label === "overlay";

    await settingsStore.load();
    initLocaleFromSettings(normalizeLocale(settingsStore.settings.locale));

    if (!settingsStore.settings.onboarding_completed && win.label === "main") {
      await router.replace("/onboarding");
    }
  } catch (error) {
    notice.value =
      error instanceof Error
        ? translateAppError(t, error.message)
        : t("errors.appStartFailed");
  }

  const listeners: Promise<UnlistenFn>[] = [
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
  ];

  // Only the main window reacts to navigation requests; module-level
  // listeners receive targeted events too, so other windows (menubar,
  // overlay, editor) would otherwise navigate themselves.
  if (getCurrentWindow().label === "main") {
    listeners.push(
      listen<string>("navigate", (event) => {
        if (event.payload) {
          router.push(event.payload);
        }
      }),
    );
  }

  unlisteners = await Promise.all(listeners);
});

onUnmounted(() => {
  unlisteners.forEach((unlisten) => unlisten());
});
</script>

<template>
  <div :class="isOverlay ? 'min-h-full bg-transparent' : 'h-full text-fg'">
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="-translate-y-2 opacity-0"
      leave-active-class="transition duration-150 ease-in"
      leave-to-class="opacity-0"
    >
      <div
        v-if="notice && !isOverlay"
        class="fixed left-1/2 top-3 z-50 flex max-w-[min(480px,calc(100vw-32px))] -translate-x-1/2 items-start gap-2.5 rounded-xl bg-material px-3.5 py-2.5 text-body text-fg shadow-popover backdrop-blur-2xl backdrop-saturate-150"
        role="alert"
      >
        <IconExclamationCircleFilled class="mt-px size-4 shrink-0 text-danger" aria-hidden="true" />
        <span>{{ notice }}</span>
      </div>
    </Transition>
    <router-view class="h-full min-h-0" />
  </div>
</template>
