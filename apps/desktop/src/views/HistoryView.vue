<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  IconCopy,
  IconCrop,
  IconDeviceDesktop,
  IconEdit,
  IconFolderOpen,
  IconLoader2,
  IconPhoto,
  IconTrash,
} from "@tabler/icons-vue";
import type { CaptureRecord } from "@better-screenshoot/shared-types";
import type { SavedCapture } from "../lib/tauri";
import {
  copyImageToClipboard,
  deleteHistoryItem,
  getHistory,
  openCaptureInEditor,
  readCaptureDataUrl,
} from "../lib/tauri";
import { useCapturePermissions } from "../composables/useCapturePermissions";
import { useSettingsStore } from "../stores/settings";
import { formatHotkey } from "../lib/format-hotkey";
import { translateAppError } from "../i18n/resolveError";
import type { AppErrorPayload } from "../i18n/resolveError";
import PendingCaptureBanner from "../components/PendingCaptureBanner.vue";
import AlertBanner from "../components/ui/AlertBanner.vue";
import AppButton from "../components/ui/AppButton.vue";
import AppKbd from "../components/ui/AppKbd.vue";
import AppContextMenu, { type ContextMenuItem } from "../components/ui/AppContextMenu.vue";
import PageScaffold from "../components/ui/PageScaffold.vue";

const { t, locale } = useI18n();
const items = ref<CaptureRecord[]>([]);
const loading = ref(true);
const openingId = ref<string | null>(null);
const error = ref<string | null>(null);
const contextMenu = ref<{ x: number; y: number; item: CaptureRecord } | null>(null);

const settingsStore = useSettingsStore();
const { permissionMessage, devBinaryPath, checkPermissions, requestPermission } =
  useCapturePermissions();

const captureActions = computed(() => [
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

const subtitle = computed(() =>
  loading.value ? undefined : t("history.count", { count: items.value.length }, items.value.length),
);

const contextMenuItems = computed<ContextMenuItem[]>(() => [
  { id: "open", label: t("history.openInEditor"), icon: IconEdit },
  { id: "copy", label: t("menubar.copy"), icon: IconCopy },
  { id: "reveal", label: t("menubar.revealInFinder"), icon: IconFolderOpen },
  { id: "delete", label: t("history.deleteCapture"), icon: IconTrash, danger: true, separatorBefore: true },
]);

const dateFormatter = computed(
  () => new Intl.DateTimeFormat(locale.value, { dateStyle: "medium", timeStyle: "short" }),
);

function formatDate(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? "" : dateFormatter.value.format(date);
}

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

async function triggerCapture(action: "capture-area" | "capture-screen") {
  await invoke("handle_capture_action", { action });
}

function openContextMenu(event: MouseEvent, item: CaptureRecord) {
  contextMenu.value = { x: event.clientX, y: event.clientY, item };
}

async function onContextMenuSelect(id: string) {
  const item = contextMenu.value?.item;
  contextMenu.value = null;
  if (!item) return;

  if (id === "open") {
    await openInEditor(item.id);
  } else if (id === "copy") {
    const dataUrl = await readCaptureDataUrl(item.file_path);
    await copyImageToClipboard(dataUrl.replace(/^data:image\/png;base64,/, ""));
  } else if (id === "reveal") {
    await revealItemInDir(item.file_path);
  } else if (id === "delete") {
    await remove(item.id);
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
  <PageScaffold :title="t('history.title')" :subtitle="subtitle">
    <template #actions>
      <AppButton
        v-for="capture in captureActions"
        :key="capture.action"
        :title="`${capture.label} (${capture.hotkey})`"
        @click="triggerCapture(capture.action)"
      >
        <component :is="capture.icon" class="size-4" stroke-width="1.75" />
        {{ capture.label }}
      </AppButton>
    </template>

    <div class="flex min-h-full flex-col px-5 pb-8 pt-1">
      <PendingCaptureBanner />

      <AlertBanner v-if="permissionMessage" tone="warning" class="mb-4">
        <p>{{ permissionMessage }}</p>
        <p v-if="devBinaryPath" class="select-text font-mono text-caption text-fg-muted">
          {{ t("history.devBinary", { path: devBinaryPath }) }}
        </p>
        <template #actions>
          <AppButton size="sm" @click="requestPermission">
            {{ t("history.openMacosPermissions") }}
          </AppButton>
        </template>
      </AlertBanner>

      <AlertBanner v-if="error && !loading" tone="danger" class="mb-4">
        {{ error }}
      </AlertBanner>

      <div v-if="loading" class="flex flex-1 items-center justify-center gap-2 text-fg-muted">
        <IconLoader2 class="size-4 animate-spin" aria-hidden="true" />
        <span>{{ t("history.loadingCaptures") }}</span>
      </div>

      <div
        v-else-if="items.length === 0"
        class="flex flex-1 flex-col items-center justify-center pb-10 text-center"
      >
        <div
          class="flex size-20 items-center justify-center rounded-[22px] bg-group shadow-[inset_0_0_0_.5px_var(--color-sep)]"
        >
          <IconPhoto class="size-10 text-fg-tertiary" stroke-width="1.25" aria-hidden="true" />
        </div>
        <h2 class="mt-5 text-title2 font-semibold text-fg">{{ t("history.empty") }}</h2>
        <p class="mt-1.5 max-w-sm text-body text-fg-muted">{{ t("history.emptyHint") }}</p>

        <ul
          class="mt-6 w-full max-w-xs overflow-hidden rounded-[var(--radius-group)] bg-group text-left shadow-[inset_0_0_0_.5px_var(--color-sep)]"
          :aria-label="t('history.captureShortcuts')"
        >
          <li
            v-for="capture in captureActions"
            :key="capture.action"
            class="relative flex h-10 items-center justify-between gap-4 px-3 before:absolute before:inset-x-3 before:top-0 before:h-px before:bg-sep first:before:hidden"
          >
            <span class="flex items-center gap-2 text-fg">
              <component :is="capture.icon" class="size-4 text-fg-muted" stroke-width="1.75" />
              {{ capture.label }}
            </span>
            <AppKbd>{{ capture.hotkey }}</AppKbd>
          </li>
        </ul>
        <RouterLink
          to="/settings/shortcuts"
          class="mt-3 text-subhead text-accent hover:underline"
        >
          {{ t("history.customizeShortcuts") }}
        </RouterLink>
      </div>

      <ul
        v-else
        class="grid grid-cols-[repeat(auto-fill,minmax(190px,1fr))] gap-x-5 gap-y-6"
      >
        <li v-for="item in items" :key="item.id" class="group min-w-0">
          <div class="relative">
            <button
              type="button"
              class="relative block aspect-[16/10] w-full overflow-hidden rounded-[10px] bg-group shadow-[0_0_0_.5px_var(--color-sep),0_2px_8px_rgb(0_0_0_/_.12)] transition-[transform,box-shadow] duration-200 ease-out hover:shadow-[0_0_0_.5px_var(--color-sep),0_8px_24px_rgb(0_0_0_/_.22)] active:scale-[.98] disabled:opacity-60"
              :class="contextMenu?.item.id === item.id ? 'shadow-[0_0_0_3px_var(--color-accent)]' : undefined"
              :disabled="openingId === item.id"
              :aria-label="
                t('history.openCaptureInEditor', {
                  width: item.width,
                  height: item.height,
                })
              "
              @click="openInEditor(item.id)"
              @contextmenu.prevent="openContextMenu($event, item)"
            >
              <img
                :src="previewSrc(item.file_path)"
                :alt="t('history.captureAlt', { id: item.id })"
                class="size-full object-cover"
                draggable="false"
                loading="lazy"
              />
              <span
                v-if="openingId === item.id"
                class="absolute inset-0 flex items-center justify-center bg-black/30"
              >
                <IconLoader2 class="size-6 animate-spin text-white" aria-hidden="true" />
              </span>
            </button>
            <button
              type="button"
              class="absolute right-2 top-2 flex size-7 items-center justify-center rounded-full bg-black/45 text-white opacity-0 backdrop-blur-md transition-opacity duration-150 hover:bg-danger focus-visible:opacity-100 group-hover:opacity-100"
              :aria-label="t('history.deleteCapture')"
              :title="t('history.deleteCapture')"
              @click.stop="remove(item.id)"
            >
              <IconTrash class="size-3.5" stroke-width="2" />
            </button>
          </div>
          <div class="mt-2 px-0.5">
            <p class="truncate text-subhead font-medium tabular-nums text-fg">
              {{ item.width }} × {{ item.height }}
            </p>
            <p class="truncate text-caption text-fg-muted">{{ formatDate(item.created_at) }}</p>
          </div>
        </li>
      </ul>
    </div>

    <AppContextMenu
      v-if="contextMenu"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :items="contextMenuItems"
      @select="onContextMenuSelect"
      @close="contextMenu = null"
    />
  </PageScaffold>
</template>
