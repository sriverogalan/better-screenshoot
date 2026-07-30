<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  IconArrowBackUp,
  IconArrowForwardUp,
  IconArrowRight,
  IconBlur,
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
  canUndo: boolean;
  canRedo: boolean;
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
  <header class="shrink-0 border-b border-sep bg-win/80 backdrop-blur-xl">
    <div class="flex items-center justify-between gap-3 px-4 py-2.5">
      <div class="min-w-0">
        <h1 class="text-[13px] font-semibold tracking-tight text-fg">
          {{ t("editor.title") }}
        </h1>
        <p
          v-if="hasCapture"
          class="truncate font-mono text-[11px] tabular-nums text-fg-muted"
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
          <IconTrash class="size-3.5" />
          {{ t("editor.copyAndDiscard") }}
        </AppButton>
        <AppButton
          variant="primary"
          :disabled="actionBusy || !canExport"
          @click="emit('copyAndSave')"
        >
          <IconDeviceFloppy class="size-3.5" />
          {{ t("editor.copyAndSave") }}
        </AppButton>
      </div>
    </div>

    <AlertBanner v-if="actionError" tone="danger" class="mx-4 mb-2">
      {{ actionError }}
    </AlertBanner>

    <div class="flex items-center gap-2 overflow-x-auto px-4 pb-2.5">
      <div
        class="inline-flex shrink-0 items-center gap-0.5 rounded-[var(--radius-control)] bg-field p-0.5"
        role="toolbar"
        :aria-label="t('editor.title')"
      >
        <button
          v-for="tool in tools"
          :key="tool.id"
          type="button"
          class="relative flex size-8 items-center justify-center rounded-[calc(var(--radius-control)-2px)] transition-colors duration-150"
          :class="
            activeTool === tool.id
              ? 'bg-elev text-accent shadow-sm'
              : 'text-fg-muted hover:bg-win/60 hover:text-fg'
          "
          :aria-label="`${tool.label} (${tool.shortcut})`"
          :aria-pressed="activeTool === tool.id"
          :title="`${tool.label} (${tool.shortcut})`"
          @click="emit('update:activeTool', tool.id)"
        >
          <component :is="tool.icon" class="size-4" stroke-width="1.75" />
        </button>
      </div>

      <div class="flex shrink-0 items-center gap-0.5">
        <button
          type="button"
          class="flex size-8 items-center justify-center rounded-[var(--radius-control)] text-fg-muted transition-colors hover:bg-elev hover:text-fg disabled:cursor-not-allowed disabled:opacity-35"
          :disabled="!canUndo"
          :aria-label="t('editor.undo')"
          :title="t('editor.undo')"
          @click="emit('undo')"
        >
          <IconArrowBackUp class="size-4" stroke-width="1.75" />
        </button>
        <button
          type="button"
          class="flex size-8 items-center justify-center rounded-[var(--radius-control)] text-fg-muted transition-colors hover:bg-elev hover:text-fg disabled:cursor-not-allowed disabled:opacity-35"
          :disabled="!canRedo"
          :aria-label="t('editor.redo')"
          :title="t('editor.redo')"
          @click="emit('redo')"
        >
          <IconArrowForwardUp class="size-4" stroke-width="1.75" />
        </button>
      </div>

      <p class="ml-auto hidden shrink-0 text-[11px] text-fg-muted xl:block">
        {{ t("editor.shortcutHint") }}
      </p>
    </div>
  </header>
</template>
