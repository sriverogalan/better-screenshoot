<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  IconArrowBackUp,
  IconArrowForwardUp,
  IconArrowRight,
  IconBlur,
  IconCopy,
  IconHighlight,
  IconPencil,
  IconPointer,
  IconRectangle,
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
  <header data-tauri-drag-region class="relative z-10 shrink-0 bg-win shadow-[0_.5px_0_var(--color-sep)]">
    <div data-tauri-drag-region class="flex h-[52px] items-center gap-3 px-3">
      <div data-tauri-drag-region class="min-w-0 flex-1 pl-1">
        <h1 data-tauri-drag-region class="truncate text-body font-semibold text-fg">
          {{ t("editor.title") }}
        </h1>
        <p
          v-if="hasCapture"
          data-tauri-drag-region
          class="truncate text-caption tabular-nums text-fg-muted"
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
        <div
          class="inline-flex items-center gap-px rounded-lg bg-hover p-[3px] shadow-[inset_0_0_0_.5px_var(--color-sep)]"
          role="toolbar"
          :aria-label="t('editor.title')"
        >
          <button
            v-for="tool in tools"
            :key="tool.id"
            type="button"
            class="flex h-[26px] w-[30px] items-center justify-center rounded-[5px] transition-[background-color,color] duration-100"
            :class="
              activeTool === tool.id
                ? 'bg-segment text-accent shadow-control'
                : 'text-fg-muted hover:bg-hover hover:text-fg'
            "
            :aria-label="`${tool.label} (${tool.shortcut})`"
            :aria-pressed="activeTool === tool.id"
            :title="`${tool.label} (${tool.shortcut})`"
            @click="emit('update:activeTool', tool.id)"
          >
            <component :is="tool.icon" class="size-[17px]" stroke-width="1.75" />
          </button>
        </div>

        <div class="flex items-center">
          <button
            type="button"
            class="flex h-[26px] w-[30px] items-center justify-center rounded-[5px] text-fg-muted transition-colors hover:bg-hover hover:text-fg disabled:opacity-30 disabled:hover:bg-transparent"
            :disabled="!canUndo"
            :aria-label="t('editor.undo')"
            :title="`${t('editor.undo')} (⌘Z)`"
            @click="emit('undo')"
          >
            <IconArrowBackUp class="size-[17px]" stroke-width="1.75" />
          </button>
          <button
            type="button"
            class="flex h-[26px] w-[30px] items-center justify-center rounded-[5px] text-fg-muted transition-colors hover:bg-hover hover:text-fg disabled:opacity-30 disabled:hover:bg-transparent"
            :disabled="!canRedo"
            :aria-label="t('editor.redo')"
            :title="`${t('editor.redo')} (⇧⌘Z)`"
            @click="emit('redo')"
          >
            <IconArrowForwardUp class="size-[17px]" stroke-width="1.75" />
          </button>
        </div>
      </div>

      <div class="flex flex-1 shrink-0 items-center justify-end gap-2">
        <AppButton
          :disabled="actionBusy || !canExport"
          @click="emit('copyAndDiscard')"
        >
          {{ t("editor.copyAndDiscard") }}
        </AppButton>
        <AppButton
          variant="primary"
          :disabled="actionBusy || !canExport"
          @click="emit('copyAndSave')"
        >
          <IconCopy class="size-3.5" stroke-width="2" />
          {{ t("editor.copyAndSave") }}
        </AppButton>
      </div>
    </div>

    <AlertBanner v-if="actionError" tone="danger" class="mx-3 mb-2.5">
      {{ actionError }}
    </AlertBanner>
  </header>
</template>
