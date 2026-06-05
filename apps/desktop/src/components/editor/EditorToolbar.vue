<script setup lang="ts">
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

const tools: { id: Tool; label: string; icon: typeof IconPointer; shortcut: string }[] = [
  { id: "select", label: "Seleccionar", icon: IconPointer, shortcut: "V" },
  { id: "arrow", label: "Flecha", icon: IconArrowRight, shortcut: "A" },
  { id: "rect", label: "Rectángulo", icon: IconRectangle, shortcut: "R" },
  { id: "text", label: "Texto", icon: IconTypography, shortcut: "T" },
  { id: "highlight", label: "Resaltar", icon: IconHighlight, shortcut: "H" },
  { id: "pen", label: "Lápiz", icon: IconPencil, shortcut: "P" },
  { id: "blur", label: "Difuminar", icon: IconBlur, shortcut: "B" },
];
</script>

<template>
  <header class="shrink-0 border-b border-border">
    <div class="flex items-center justify-between gap-4 px-4 py-3">
      <div class="min-w-0">
        <h1 class="text-sm font-medium">Editor</h1>
        <p
          v-if="hasCapture"
          class="truncate text-xs text-text-muted"
        >
          {{ imageWidth }} × {{ imageHeight }} px · {{ zoomPercent }}%
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
          Copiar y descartar
        </button>
        <button
          type="button"
          class="inline-flex items-center gap-1.5 rounded-lg bg-accent px-3 py-2 text-sm text-white hover:bg-accent-hover disabled:opacity-50"
          :disabled="actionBusy || !canExport"
          @click="emit('copyAndSave')"
        >
          <IconDeviceFloppy class="size-4" />
          Copiar y guardar
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
        Deshacer
      </button>
      <button
        type="button"
        class="shrink-0 rounded-lg px-3 py-1.5 text-sm hover:bg-surface-raised"
        @click="emit('redo')"
      >
        Rehacer
      </button>

      <p class="ml-auto hidden shrink-0 text-xs text-text-muted lg:block">
        <IconClipboardCopy class="mr-1 inline size-3.5" />
        V seleccionar · A flecha · R rectángulo · T texto
      </p>
    </div>
  </header>
</template>
