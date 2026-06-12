<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  IconArrowRight,
  IconBlur,
  IconClipboardCopy,
  IconDeviceFloppy,
  IconHighlight,
  IconPencil,
  IconPointer,
  IconRectangle,
  IconTrash,
  IconTypography,
} from "@tabler/icons-vue";
import type { Tool } from "../../lib/editor/types";

defineProps<{
  activeTool: Tool;
  actionBusy: boolean;
  actionError: string | null;
  canExport: boolean;
  imageWidth: number;
  imageHeight: number;
  zoomPercent: number;
  hasCapture: boolean;
}>();

const emit = defineEmits<{
  "update:activeTool": [tool: Tool];
  undo: [];
  redo: [];
  copyAndDiscard: [];
  copyAndSave: [];
}>();

const { t } = useI18n();

const tools = computed(() => [
  { id: "select" as const, label: t("editor.tools.select"), icon: IconPointer, shortcut: "V" },
  { id: "arrow" as const, label: t("editor.tools.arrow"), icon: IconArrowRight, shortcut: "A" },
  { id: "rect" as const, label: t("editor.tools.rect"), icon: IconRectangle, shortcut: "R" },
  { id: "text" as const, label: t("editor.tools.text"), icon: IconTypography, shortcut: "T" },
  { id: "highlight" as const, label: t("editor.tools.highlight"), icon: IconHighlight, shortcut: "H" },
  { id: "pen" as const, label: t("editor.tools.pen"), icon: IconPencil, shortcut: "P" },
  { id: "blur" as const, label: t("editor.tools.blur"), icon: IconBlur, shortcut: "B" },
]);
</script>

<template>
  <header class="shrink-0 border-b border-border">
    <div class="flex items-center justify-between gap-4 px-4 py-3">
      <div class="min-w-0">
        <h1 class="text-sm font-medium">{{ t("editor.title") }}</h1>
        <p
          v-if="hasCapture"
          class="truncate text-xs text-text-muted"
        >
          {{
            t("editor.dimensions", {
              width: imageWidth,
              height: imageHeight,
              zoom: zoomPercent,
            })
          }}
        </p>
      </div>

      <div class="flex shrink-0 items-center gap-2">
        <button
          type="button"
          class="inline-flex items-center gap-1.5 rounded-lg border border-border bg-surface-raised px-3 py-2 text-sm hover:bg-border disabled:opacity-50"
          :disabled="actionBusy || !canExport"
          @click="emit('copyAndDiscard')"
        >
          <IconTrash class="size-4" />
          {{ t("editor.copyAndDiscard") }}
        </button>
        <button
          type="button"
          class="inline-flex items-center gap-1.5 rounded-lg bg-accent px-3 py-2 text-sm text-white hover:bg-accent-hover disabled:opacity-50"
          :disabled="actionBusy || !canExport"
          @click="emit('copyAndSave')"
        >
          <IconDeviceFloppy class="size-4" />
          {{ t("editor.copyAndSave") }}
        </button>
      </div>
    </div>

    <p
      v-if="actionError"
      class="border-t border-red-500/30 bg-red-950/40 px-4 py-2 text-xs text-red-200"
      role="alert"
    >
      {{ actionError }}
    </p>

    <div class="flex items-center gap-1 overflow-x-auto border-t border-border/60 px-4 py-2">
      <button
        v-for="tool in tools"
        :key="tool.id"
        type="button"
        class="shrink-0 rounded-lg p-2 hover:bg-surface-raised"
        :class="{ 'bg-surface-raised text-accent': activeTool === tool.id }"
        :aria-label="`${tool.label} (${tool.shortcut})`"
        :title="`${tool.label} (${tool.shortcut})`"
        @click="emit('update:activeTool', tool.id)"
      >
        <component :is="tool.icon" class="size-4" />
      </button>

      <span class="mx-1 h-5 w-px shrink-0 bg-border" />

      <button
        type="button"
        class="shrink-0 rounded-lg px-3 py-1.5 text-sm hover:bg-surface-raised"
        @click="emit('undo')"
      >
        {{ t("editor.undo") }}
      </button>
      <button
        type="button"
        class="shrink-0 rounded-lg px-3 py-1.5 text-sm hover:bg-surface-raised"
        @click="emit('redo')"
      >
        {{ t("editor.redo") }}
      </button>

      <p class="ml-auto hidden shrink-0 text-xs text-text-muted lg:block">
        <IconClipboardCopy class="mr-1 inline size-3.5" />
        {{ t("editor.shortcutHint") }}
      </p>
    </div>
  </header>
</template>
