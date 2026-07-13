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
import AppButton from "../ui/AppButton.vue";
import AlertBanner from "../ui/AlertBanner.vue";

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
  <header class="shrink-0 border-b border-sep">
    <div class="flex items-center justify-between gap-4 px-4 py-3">
      <div class="min-w-0">
        <h1 class="text-sm font-medium text-fg">{{ t("editor.title") }}</h1>
        <p
          v-if="hasCapture"
          class="truncate text-xs text-fg-muted"
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
        <AppButton
          variant="secondary"
          :disabled="actionBusy || !canExport"
          @click="emit('copyAndDiscard')"
        >
          <IconTrash class="size-4" />
          {{ t("editor.copyAndDiscard") }}
        </AppButton>
        <AppButton
          variant="primary"
          :disabled="actionBusy || !canExport"
          @click="emit('copyAndSave')"
        >
          <IconDeviceFloppy class="size-4" />
          {{ t("editor.copyAndSave") }}
        </AppButton>
      </div>
    </div>

    <AlertBanner v-if="actionError" tone="danger" class="mx-4 mb-2">
      {{ actionError }}
    </AlertBanner>

    <div class="flex items-center gap-1 overflow-x-auto border-t border-sep/60 px-4 py-2">
      <button
        v-for="tool in tools"
        :key="tool.id"
        type="button"
        class="shrink-0 rounded-lg p-2 hover:bg-elev"
        :class="
          activeTool === tool.id
            ? 'bg-accent/15 text-accent'
            : 'text-fg-muted hover:text-fg'
        "
        :aria-label="`${tool.label} (${tool.shortcut})`"
        :title="`${tool.label} (${tool.shortcut})`"
        @click="emit('update:activeTool', tool.id)"
      >
        <component :is="tool.icon" class="size-4" />
      </button>

      <span class="mx-1 h-5 w-px shrink-0 bg-sep" />

      <button
        type="button"
        class="shrink-0 rounded-lg px-3 py-1.5 text-sm text-fg-muted hover:bg-elev hover:text-fg"
        @click="emit('undo')"
      >
        {{ t("editor.undo") }}
      </button>
      <button
        type="button"
        class="shrink-0 rounded-lg px-3 py-1.5 text-sm text-fg-muted hover:bg-elev hover:text-fg"
        @click="emit('redo')"
      >
        {{ t("editor.redo") }}
      </button>

      <p class="ml-auto hidden shrink-0 text-xs text-fg-muted lg:block">
        <IconClipboardCopy class="mr-1 inline size-3.5" />
        {{ t("editor.shortcutHint") }}
      </p>
    </div>
  </header>
</template>
