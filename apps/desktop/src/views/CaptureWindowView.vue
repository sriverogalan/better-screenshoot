<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { WindowInfo } from "@better-screenshoot/shared-types";
import { captureWindow, listWindows } from "../lib/tauri";
const windows = ref<WindowInfo[]>([]);
const loading = ref(true);
const capturing = ref<number | null>(null);

async function load() {
  loading.value = true;
  try {
    windows.value = await listWindows();
  } catch {
    windows.value = [];
  } finally {
    loading.value = false;
  }
}

async function capture(id: number) {
  capturing.value = id;
  try {
    await captureWindow(id);
    await getCurrentWindow().hide();
  } finally {
    capturing.value = null;
  }
}

onMounted(load);
</script>

<template>
  <div class="flex min-h-full flex-col p-6">
    <header class="mb-6">
      <h1 class="text-lg font-semibold">Capture window</h1>
      <p class="mt-1 text-sm text-text-muted">Select the window to capture</p>
    </header>

    <main class="flex-1">
      <p v-if="loading" class="text-sm text-text-muted">Searching for windows…</p>
      <p v-else-if="windows.length === 0" class="text-sm text-text-muted">
        No windows found.
      </p>
      <ul v-else class="space-y-2">
        <li v-for="win in windows" :key="win.id">
          <button
            type="button"
            class="flex w-full items-center justify-between rounded-xl border border-border bg-surface-raised px-4 py-3 text-left hover:border-accent"
            :disabled="capturing === win.id"
            @click="capture(win.id)"
          >
            <span>
              <span class="block text-sm font-medium">{{ win.title || "Untitled" }}</span>
              <span class="text-xs text-text-muted">{{ win.app_name }}</span>
            </span>
            <span class="text-xs text-text-muted">{{ win.width }}×{{ win.height }}</span>
          </button>
        </li>
      </ul>
    </main>
  </div>
</template>
