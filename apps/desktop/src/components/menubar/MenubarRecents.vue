<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { CaptureRecord } from "@better-screenshoot/shared-types";
import { IconPhotoOff, IconCopy, IconFolderOpen, IconTrash } from "@tabler/icons-vue";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { startDrag } from "@crabnebula/tauri-plugin-drag";
import {
  getHistory,
  openCaptureInEditor,
  readCaptureDataUrl,
  copyImageToClipboard,
  deleteHistoryItem,
} from "../../lib/tauri";

const { t } = useI18n();
const recents = ref<CaptureRecord[]>([]);
const contextMenu = ref<{ x: number; y: number; item: CaptureRecord } | null>(null);
const unlisteners: UnlistenFn[] = [];

async function loadRecents() {
  try {
    const all = await getHistory(6);
    recents.value = all.slice(0, 6);
  } catch {
    recents.value = [];
  }
}

function onGlobalKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && contextMenu.value) {
    e.stopPropagation();
    closeContextMenu();
  }
}

onMounted(async () => {
  await loadRecents();
  unlisteners.push(
    await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused) {
        void loadRecents();
      }
    }),
  );
  unlisteners.push(await listen("history-changed", () => void loadRecents()));
  window.addEventListener("keydown", onGlobalKeydown, true);
});

onUnmounted(() => {
  unlisteners.forEach((unlisten) => unlisten());
  window.removeEventListener("keydown", onGlobalKeydown, true);
});

function previewSrc(path: string): string {
  return convertFileSrc(path);
}

async function openInEditor(item: CaptureRecord) {
  await getCurrentWindow().hide();
  await openCaptureInEditor(item.id);
}

function openContextMenu(event: MouseEvent, item: CaptureRecord) {
  const menuWidth = 168;
  const menuHeight = 128;
  contextMenu.value = {
    x: Math.min(event.clientX, window.innerWidth - menuWidth - 8),
    y: Math.min(event.clientY, window.innerHeight - menuHeight - 8),
    item,
  };
}

function closeContextMenu() {
  contextMenu.value = null;
}

async function copyToClipboard(item: CaptureRecord) {
  closeContextMenu();
  const dataUrl = await readCaptureDataUrl(item.file_path);
  await copyImageToClipboard(dataUrl.replace(/^data:image\/png;base64,/, ""));
}

async function revealInFinder(item: CaptureRecord) {
  closeContextMenu();
  await revealItemInDir(item.file_path);
}

async function deleteCapture(item: CaptureRecord) {
  closeContextMenu();
  await deleteHistoryItem(item.id);
  await loadRecents();
}

function onThumbnailDragStart(item: CaptureRecord) {
  void startDrag({ item: [item.file_path], icon: item.file_path });
}
</script>

<template>
  <div class="border-t border-sep px-2.5 py-2.5">
    <p class="mb-2 text-xs font-semibold uppercase tracking-wider text-fg-muted">
      {{ t("menubar.recentCaptures") }}
    </p>

    <div v-if="recents.length === 0" class="flex flex-col items-center gap-1.5 py-3">
      <IconPhotoOff class="size-5 text-fg-muted opacity-60" />
      <p class="text-xs text-fg-muted">{{ t("history.empty") }}</p>
    </div>

    <div v-else class="grid grid-cols-3 gap-1.5">
      <button
        v-for="item in recents"
        :key="item.id"
        type="button"
        draggable="true"
        class="aspect-video overflow-hidden rounded-lg bg-elev transition-opacity hover:opacity-80"
        :title="t('history.openCaptureInEditor', { width: item.width, height: item.height })"
        @click="openInEditor(item)"
        @contextmenu.prevent="openContextMenu($event, item)"
        @dragstart.prevent="onThumbnailDragStart(item)"
      >
        <img
          :src="previewSrc(item.file_path)"
          :alt="t('history.captureAlt', { id: item.id })"
          draggable="false"
          class="h-full w-full object-cover"
        />
      </button>
    </div>

    <div v-if="contextMenu" class="fixed inset-0 z-40" @click="closeContextMenu" @contextmenu.prevent="closeContextMenu" />

    <div
      v-if="contextMenu"
      class="fixed z-50 min-w-[168px] rounded-lg border border-sep bg-elev p-1 shadow-window"
      :style="{ top: `${contextMenu.y}px`, left: `${contextMenu.x}px` }"
    >
      <button
        type="button"
        class="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-sm text-fg hover:bg-win transition-colors"
        @click="copyToClipboard(contextMenu.item)"
      >
        <IconCopy class="size-4 shrink-0" />
        {{ t("menubar.copy") }}
      </button>

      <button
        type="button"
        class="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-sm text-fg hover:bg-win transition-colors"
        @click="revealInFinder(contextMenu.item)"
      >
        <IconFolderOpen class="size-4 shrink-0" />
        {{ t("menubar.revealInFinder") }}
      </button>

      <div class="my-1 border-t border-sep"></div>

      <button
        type="button"
        class="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-sm text-danger hover:bg-win transition-colors"
        @click="deleteCapture(contextMenu.item)"
      >
        <IconTrash class="size-4 shrink-0" />
        {{ t("history.deleteCapture") }}
      </button>
    </div>
  </div>
</template>
