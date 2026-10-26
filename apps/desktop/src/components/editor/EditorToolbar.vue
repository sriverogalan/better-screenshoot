<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  IconArrowBackUp,
  IconArrowForwardUp,
  IconArrowRight,
  IconBlur,
  IconCopy,
  IconFileZip,
  IconHighlight,
  IconPencil,
  IconPointer,
  IconRectangle,
  IconTypography,
} from "@tabler/icons-vue";
import type { Tool } from "../../lib/editor/types";
import { formatBytes } from "../../lib/format-bytes";
import AppButton from "../ui/AppButton.vue";
import AppTooltip from "../ui/AppTooltip.vue";
import AlertBanner from "../ui/AlertBanner.vue";

const props = defineProps<{
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
  compress: boolean;
  pngBytes: number | null;
  jpegBytes: number | null;
  sizesBusy: boolean;
}>();

const emit = defineEmits<{
  "update:activeTool": [tool: Tool];
  "update:compress": [value: boolean];
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

const sizeLabel = computed(() => {
  if (props.pngBytes == null || props.jpegBytes == null) {
    return props.sizesBusy ? t("editor.compress.estimating") : null;
  }
  return t("editor.compress.sizes", {
    png: formatBytes(props.pngBytes),
    jpeg: formatBytes(props.jpegBytes),
  });
});
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
          <AppTooltip
            v-for="tool in tools"
            :key="tool.id"
            :label="tool.label"
            :shortcut="tool.shortcut"
          >
            <button
              type="button"
              class="flex h-[26px] w-[30px] items-center justify-center rounded-[5px] transition-[background-color,color] duration-100"
              :class="
                activeTool === tool.id
                  ? 'bg-segment text-accent shadow-control'
                  : 'text-fg-muted hover:bg-hover hover:text-fg'
              "
              :aria-label="`${tool.label} (${tool.shortcut})`"
              :aria-pressed="activeTool === tool.id"
              @click="emit('update:activeTool', tool.id)"
            >
              <component :is="tool.icon" class="size-[17px]" stroke-width="1.75" />
            </button>
          </AppTooltip>
        </div>

        <div class="flex items-center">
          <AppTooltip :label="t('editor.undo')" shortcut="⌘Z" :disabled="!canUndo">
            <button
              type="button"
              class="flex h-[26px] w-[30px] items-center justify-center rounded-[5px] text-fg-muted transition-colors hover:bg-hover hover:text-fg disabled:opacity-30 disabled:hover:bg-transparent"
              :disabled="!canUndo"
              :aria-label="t('editor.undo')"
              @click="emit('undo')"
            >
              <IconArrowBackUp class="size-[17px]" stroke-width="1.75" />
            </button>
          </AppTooltip>
          <AppTooltip :label="t('editor.redo')" shortcut="⇧⌘Z" :disabled="!canRedo">
            <button
              type="button"
              class="flex h-[26px] w-[30px] items-center justify-center rounded-[5px] text-fg-muted transition-colors hover:bg-hover hover:text-fg disabled:opacity-30 disabled:hover:bg-transparent"
              :disabled="!canRedo"
              :aria-label="t('editor.redo')"
              @click="emit('redo')"
            >
              <IconArrowForwardUp class="size-[17px]" stroke-width="1.75" />
            </button>
          </AppTooltip>
        </div>
      </div>

      <div class="flex flex-1 shrink-0 items-center justify-end gap-2">
        <AppTooltip
          :label="compress ? t('editor.compress.on') : t('editor.compress.off')"
          :disabled="!canExport"
        >
          <button
            type="button"
            class="inline-flex h-[26px] items-center gap-1.5 rounded-[5px] px-2 text-caption transition-colors disabled:opacity-30"
            :class="
              compress
                ? 'bg-segment text-accent shadow-control'
                : 'text-fg-muted hover:bg-hover hover:text-fg'
            "
            :disabled="actionBusy || !canExport"
            :aria-pressed="compress"
            :aria-label="t('editor.compress.label')"
            @click="emit('update:compress', !compress)"
          >
            <IconFileZip class="size-[15px]" stroke-width="1.75" />
            <span class="hidden whitespace-nowrap sm:inline">{{ t("editor.compress.label") }}</span>
          </button>
        </AppTooltip>
        <span
          v-if="sizeLabel && canExport"
          class="hidden max-w-[160px] truncate text-caption tabular-nums text-fg-muted lg:inline"
          :title="sizeLabel"
        >
          {{ sizeLabel }}
        </span>
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
