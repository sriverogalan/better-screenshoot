<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { convertFileSrc } from "@tauri-apps/api/core";
import { IconTrash } from "@tabler/icons-vue";
import type { CaptureRecord } from "@better-screenshoot/shared-types";
import type { SavedCapture } from "../lib/tauri";
import { deleteHistoryItem, getHistory, openCaptureInEditor } from "../lib/tauri";
import { useCapturePermissions } from "../composables/useCapturePermissions";
import { useSettingsStore } from "../stores/settings";
import { formatHotkey } from "../lib/format-hotkey";
import { translateAppError } from "../i18n/resolveError";
import type { AppErrorPayload } from "../i18n/resolveError";
import PendingCaptureBanner from "../components/PendingCaptureBanner.vue";

const { t } = useI18n();
const items = ref<CaptureRecord[]>([]);
const loading = ref(true);
const openingId = ref<string | null>(null);
const error = ref<string | null>(null);

const settingsStore = useSettingsStore();
const { permissionMessage, devBinaryPath, checkPermissions, requestPermission } =
  useCapturePermissions();

const captureShortcuts = computed(() => [
  { label: t("history.captureRegion"), hotkey: settingsStore.settings.hotkeys.capture_area },
  { label: t("history.captureScreen"), hotkey: settingsStore.settings.hotkeys.capture_screen },
  { label: t("history.captureWindow"), hotkey: settingsStore.settings.hotkeys.capture_window },
]);

async function load() {
  loading.value = true;
  error.value = null;
  try {
    items.value = await getHistory();
  } catch (err) {
    error.value =
      err instanceof Error
        ? translateAppError(t, err.message)
        : t("errors.loadHistoryFailed");
  } finally {
    loading.value = false;
  }
}

async function remove(id: string) {
  await deleteHistoryItem(id);
  items.value = items.value.filter((item) => item.id !== id);
}

function previewSrc(path: string) {
  return convertFileSrc(path);
}

async function openInEditor(id: string) {
  openingId.value = id;
  try {
    await openCaptureInEditor(id);
  } catch (err) {
    error.value =
      err instanceof Error
        ? translateAppError(t, err.message)
        : t("errors.openEditorFailed");
  } finally {
    openingId.value = null;
  }
}

let unlisteners: UnlistenFn[] = [];

onMounted(async () => {
  await Promise.all([load(), checkPermissions(), settingsStore.load()]);
  unlisteners = await Promise.all([
    listen("history-changed", () => {
      void load();
    }),
    listen<SavedCapture>("capture-complete", () => {
      void load();
    }),
    listen<string | AppErrorPayload>("capture-error", (event) => {
      error.value = translateAppError(t, event.payload);
    }),
  ]);
});

onUnmounted(() => {
  unlisteners.forEach((unlisten) => unlisten());
});
</script>

<template>
  <div class="flex min-h-full flex-col p-6">
    <header class="mb-6">
      <h1 class="text-lg font-semibold">{{ t("history.title") }}</h1>
      <p class="mt-1 text-sm text-text-muted">{{ t("history.subtitle") }}</p>
    </header>

    <PendingCaptureBanner />

    <div
      v-if="permissionMessage"
      class="mb-4 rounded-xl border border-amber-500/40 bg-amber-950/40 px-4 py-3 text-sm text-amber-100"
      role="status"
    >
      <p>{{ permissionMessage }}</p>
      <p v-if="devBinaryPath" class="mt-2 font-mono text-xs text-amber-200/80">
        {{ t("history.devBinary", { path: devBinaryPath }) }}
      </p>
      <button
        type="button"
        class="mt-3 rounded-lg bg-amber-600/80 px-3 py-1.5 text-xs font-medium text-white hover:bg-amber-600"
        @click="requestPermission"
      >
        {{ t("history.openMacosPermissions") }}
      </button>
    </div>

    <section
      class="mb-6 rounded-xl border border-border bg-surface-raised p-4"
      aria-labelledby="capture-shortcuts-heading"
    >
      <h2 id="capture-shortcuts-heading" class="text-sm font-medium">
        {{ t("history.captureShortcuts") }}
      </h2>
      <p class="mt-1 text-xs text-text-muted">
        {{ t("history.captureShortcutsHint") }}
      </p>
      <ul class="mt-3 space-y-2">
        <li
          v-for="item in captureShortcuts"
          :key="item.label"
          class="flex items-center justify-between gap-4 text-sm"
        >
          <span class="text-text-muted">{{ item.label }}</span>
          <kbd
            class="rounded-md border border-border bg-surface px-2 py-0.5 font-mono text-xs text-text"
          >
            {{ formatHotkey(item.hotkey) }}
          </kbd>
        </li>
      </ul>
      <RouterLink
        to="/settings"
        class="mt-3 inline-block text-xs text-accent hover:text-accent-hover"
      >
        {{ t("history.customizeShortcuts") }}
      </RouterLink>
    </section>

    <p v-if="loading" class="text-sm text-text-muted">{{ t("history.loadingCaptures") }}</p>
    <p v-else-if="error" class="text-sm text-red-400">{{ error }}</p>
    <div v-else-if="items.length === 0" class="space-y-3 text-sm text-text-muted">
      <p>{{ t("history.empty") }}</p>
      <p class="text-xs">{{ t("history.emptyHint") }}</p>
    </div>
    <ul v-else class="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4">
      <li
        v-for="item in items"
        :key="item.id"
        class="group overflow-hidden rounded-xl border border-border bg-surface-raised"
      >
        <button
          type="button"
          class="block w-full text-left disabled:opacity-50"
          :disabled="openingId === item.id"
          :aria-label="
            t('history.openCaptureInEditor', {
              width: item.width,
              height: item.height,
            })
          "
          @click="openInEditor(item.id)"
        >
          <img
            :src="previewSrc(item.file_path)"
            :alt="t('history.captureAlt', { id: item.id })"
            class="aspect-video w-full object-cover"
          />
        </button>
        <div
          class="flex items-center justify-between px-3 py-2 text-xs text-text-muted"
        >
          <span>{{ item.width }}×{{ item.height }}</span>
          <button
            type="button"
            class="rounded p-1 opacity-0 transition group-hover:opacity-100 hover:bg-border hover:text-red-400"
            :aria-label="t('history.deleteCapture')"
            @click.stop="remove(item.id)"
          >
            <IconTrash class="size-4" />
          </button>
        </div>
      </li>
    </ul>
  </div>
</template>
