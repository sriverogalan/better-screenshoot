<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { convertFileSrc } from "@tauri-apps/api/core";
import { completeAreaCapture } from "../lib/tauri";
import type { Region } from "@better-screenshoot/shared-types";
import { translateAppError } from "../i18n/resolveError";
import type { AppErrorPayload } from "../i18n/resolveError";

interface OverlayPreview {
  preview_path: string;
  width: number;
  height: number;
  source_width: number;
  source_height: number;
  display_id: number;
  scale_factor: number;
}

const { t } = useI18n();
const preview = ref<OverlayPreview | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);
const viewport = ref({ width: window.innerWidth, height: window.innerHeight });
const start = ref<{ x: number; y: number } | null>(null);
const current = ref<{ x: number; y: number } | null>(null);
const dragging = ref(false);

const selection = computed(() => {
  if (!start.value || !current.value) return null;
  const x = Math.min(start.value.x, current.value.x);
  const y = Math.min(start.value.y, current.value.y);
  const width = Math.abs(current.value.x - start.value.x);
  const height = Math.abs(current.value.y - start.value.y);
  return { x, y, width, height };
});

const dimPanels = computed(() => {
  if (!selection.value) return null;
  const { x, y, width, height } = selection.value;
  const vw = viewport.value.width;
  const vh = viewport.value.height;
  return {
    top: { top: 0, left: 0, width: vw, height: y },
    bottom: { top: y + height, left: 0, width: vw, height: vh - y - height },
    left: { top: y, left: 0, width: x, height },
    right: { top: y, left: x + width, width: vw - x - width, height },
  };
});

const selectionStyle = computed(() => {
  if (!selection.value) return {};
  const { x, y, width, height } = selection.value;
  return {
    left: `${x}px`,
    top: `${y}px`,
    width: `${width}px`,
    height: `${height}px`,
  };
});

const sizeLabel = computed(() => {
  if (!selection.value || !preview.value || selection.value.width < 2) return "";
  const scaleX = preview.value.source_width / viewport.value.width;
  const scaleY = preview.value.source_height / viewport.value.height;
  const w = Math.round(selection.value.width * scaleX);
  const h = Math.round(selection.value.height * scaleY);
  return `${w} × ${h}`;
});

function panelStyle(panel: { top: number; left: number; width: number; height: number }) {
  return {
    top: `${panel.top}px`,
    left: `${panel.left}px`,
    width: `${Math.max(panel.width, 0)}px`,
    height: `${Math.max(panel.height, 0)}px`,
  };
}

function toImageRegion(sel: { x: number; y: number; width: number; height: number }): Region {
  if (!preview.value) {
    return { x: 0, y: 0, width: 0, height: 0 };
  }
  const scaleX = preview.value.source_width / viewport.value.width;
  const scaleY = preview.value.source_height / viewport.value.height;
  return {
    x: Math.round(sel.x * scaleX),
    y: Math.round(sel.y * scaleY),
    width: Math.round(sel.width * scaleX),
    height: Math.round(sel.height * scaleY),
  };
}

function onPreviewImageLoad() {
  loading.value = false;
}

function onPreviewImageError() {
  error.value = t("errors.loadPreviewFailed");
  loading.value = false;
}

function onPointerDown(event: PointerEvent) {
  if (!preview.value || loading.value) return;
  dragging.value = true;
  start.value = { x: event.clientX, y: event.clientY };
  current.value = { x: event.clientX, y: event.clientY };
}

function onPointerMove(event: PointerEvent) {
  if (!dragging.value) return;
  current.value = { x: event.clientX, y: event.clientY };
}

async function onPointerUp() {
  if (!dragging.value || !selection.value || !preview.value) {
    dragging.value = false;
    return;
  }

  dragging.value = false;
  const { width, height } = selection.value;
  if (width < 6 || height < 6) {
    await closeOverlay();
    return;
  }

  const region = toImageRegion(selection.value);

  try {
    await completeAreaCapture(preview.value.display_id, region);
  } catch (err) {
    error.value =
      err instanceof Error
        ? translateAppError(t, err.message)
        : t("errors.completeCaptureFailed");
  } finally {
    if (!error.value) {
      await closeOverlay();
    }
  }
}

async function closeOverlay() {
  const overlay = getCurrentWindow();
  await overlay.setFullscreen(false);
  await overlay.hide();
  preview.value = null;
  start.value = null;
  current.value = null;
  loading.value = true;
  error.value = null;
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    void closeOverlay();
  }
}

function updateViewport() {
  viewport.value = { width: window.innerWidth, height: window.innerHeight };
}

let unlisteners: UnlistenFn[] = [];

onMounted(async () => {
  document.documentElement.classList.add("overlay-mode");
  updateViewport();
  window.addEventListener("resize", updateViewport);

  unlisteners = await Promise.all([
    listen("overlay-loading", () => {
      loading.value = true;
      error.value = null;
    }),
    listen<OverlayPreview>("overlay-preview", (event) => {
      preview.value = event.payload;
      loading.value = true;
    }),
    listen<string | AppErrorPayload>("overlay-error", (event) => {
      error.value = translateAppError(t, event.payload);
      loading.value = false;
    }),
  ]);

  await getCurrentWindow().setFocus();
});

onUnmounted(() => {
  document.documentElement.classList.remove("overlay-mode");
  window.removeEventListener("resize", updateViewport);
  unlisteners.forEach((unlisten) => unlisten());
});
</script>

<template>
  <div
    class="fixed inset-0 cursor-crosshair select-none overflow-hidden bg-transparent"
    role="application"
    :aria-label="t('overlay.regionSelector')"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @keydown="onKeyDown"
    tabindex="0"
  >
    <img
      v-if="preview"
      :src="convertFileSrc(preview.preview_path)"
      alt=""
      class="pointer-events-none absolute inset-0 h-full w-full object-fill"
      draggable="false"
      @load="onPreviewImageLoad"
      @error="onPreviewImageError"
    />

    <template v-if="selection && dimPanels">
      <div
        v-for="(panel, key) in dimPanels"
        :key="key"
        class="pointer-events-none absolute bg-black/60 backdrop-blur-[1px]"
        :style="panelStyle(panel)"
      />
    </template>

    <div
      v-if="selection && selection.width > 0 && selection.height > 0"
      class="pointer-events-none absolute border-2 border-white shadow-[0_0_0_1px_rgba(0,0,0,0.35)]"
      :style="selectionStyle"
    >
      <span
        class="absolute -top-8 left-0 rounded-md bg-black/80 px-2.5 py-1 font-mono text-xs text-white shadow-lg"
      >
        {{ sizeLabel }}
      </span>
      <span class="absolute -left-1 -top-1 size-2 rounded-full bg-white shadow" />
      <span class="absolute -right-1 -top-1 size-2 rounded-full bg-white shadow" />
      <span class="absolute -bottom-1 -left-1 size-2 rounded-full bg-white shadow" />
      <span class="absolute -bottom-1 -right-1 size-2 rounded-full bg-white shadow" />
    </div>

    <div
      v-if="loading"
      class="pointer-events-none absolute inset-0 flex items-center justify-center bg-black/40"
    >
      <p class="rounded-full bg-black/70 px-5 py-2.5 text-sm text-white">
        {{ t("overlay.preparing") }}
      </p>
    </div>

    <p
      v-if="error"
      class="pointer-events-none absolute inset-x-0 top-8 mx-auto max-w-md rounded-xl bg-red-950/90 px-4 py-3 text-center text-sm text-red-100"
    >
      {{ error }}
    </p>

    <p
      v-if="!loading && preview && !error"
      class="pointer-events-none fixed bottom-6 left-1/2 -translate-x-1/2 rounded-full bg-black/70 px-5 py-2.5 text-sm text-white/90 shadow-lg"
    >
      {{ t("overlay.hint") }}
    </p>
  </div>
</template>
