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
  const menuWidth = 200;
  const menuHeight = 96;
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
  <div class="px-2.5 pb-2">
    <p class="px-1 pb-1.5 text-caption font-semibold text-fg-muted">
      {{ t("menubar.recentCaptures") }}
    </p>

    <div
      v-if="recents.length === 0"
      class="flex flex-col items-center gap-1.5 rounded-xl bg-tile py-5 shadow-[inset_0_0_0_.5px_var(--color-sep)]"
    >
      <IconPhotoOff class="size-5 text-fg-tertiary" stroke-width="1.5" />
      <p class="text-subhead text-fg-muted">{{ t("history.empty") }}</p>
    </div>

    <div v-else class="grid grid-cols-3 gap-1.5">
      <button
        v-for="item in recents"
        :key="item.id"
        type="button"
        draggable="true"
        class="aspect-[16/10] overflow-hidden rounded-lg bg-tile shadow-[0_0_0_.5px_var(--color-sep)] outline-none transition-[transform,box-shadow] duration-100 hover:shadow-[0_0_0_2px_var(--color-accent)] focus-visible:shadow-[0_0_0_2px_var(--color-accent)] active:scale-[.96]"
        :class="contextMenu?.item.id === item.id ? 'shadow-[0_0_0_2px_var(--color-accent)]' : undefined"
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
      class="fixed z-50 w-[200px] rounded-[9px] bg-material p-[5px] text-body shadow-popover backdrop-blur-2xl"
      :style="{ top: `${contextMenu.y}px`, left: `${contextMenu.x}px` }"
      role="menu"
    >
      <button
        type="button"
        role="menuitem"
        class="flex h-6 w-full items-center gap-2 rounded-[5px] px-2 text-fg hover:bg-accent hover:text-white"
        @click="copyToClipboard(contextMenu.item)"
      >
        <IconCopy class="size-4 shrink-0" stroke-width="1.75" />
        {{ t("menubar.copy") }}
      </button>

      <button
        type="button"
        role="menuitem"
        class="flex h-6 w-full items-center gap-2 rounded-[5px] px-2 text-fg hover:bg-accent hover:text-white"
        @click="revealInFinder(contextMenu.item)"
      >
        <IconFolderOpen class="size-4 shrink-0" stroke-width="1.75" />
        {{ t("menubar.revealInFinder") }}
      </button>

      <div class="mx-2 my-[5px] h-px bg-sep" role="separator" />

      <button
        type="button"
        role="menuitem"
        class="flex h-6 w-full items-center gap-2 rounded-[5px] px-2 text-danger hover:bg-accent hover:text-white"
        @click="deleteCapture(contextMenu.item)"
      >
        <IconTrash class="size-4 shrink-0" stroke-width="1.75" />
        {{ t("history.deleteCapture") }}
      </button>
    </div>
  </div>
</template>
