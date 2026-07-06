<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { CaptureRecord } from "@better-screenshoot/shared-types";
import { getHistory } from "../../lib/tauri";

const { t } = useI18n();
const recents = ref<CaptureRecord[]>([]);

onMounted(async () => {
  try {
    const all = await getHistory(6);
    recents.value = all.slice(0, 6);
  } catch {
    recents.value = [];
  }
});

function previewSrc(path: string): string {
  return convertFileSrc(path);
}
</script>

<template>
  <div class="px-3 py-2 border-b border-sep">
    <p class="mb-2 text-xs font-semibold uppercase tracking-wider text-fg-muted">
      {{ t("menubar.recentCaptures") }}
    </p>

    <p v-if="recents.length === 0" class="text-xs text-fg-muted text-center py-2">
      {{ t("history.empty") }}
    </p>

    <div v-else class="grid grid-cols-3 gap-1.5">
      <div
        v-for="item in recents"
        :key="item.id"
        class="aspect-video overflow-hidden rounded-lg bg-elev"
      >
        <img
          :src="previewSrc(item.file_path)"
          :alt="t('history.captureAlt', { id: item.id })"
          class="h-full w-full object-cover"
        />
      </div>
    </div>
  </div>
</template>
