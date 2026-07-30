<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { IconCheck } from "@tabler/icons-vue";
import type { EditorStyle, Tool } from "../../lib/editor/types";
import {
  COLOR_PRESETS,
  FONT_SIZES,
  STROKE_WIDTHS,
} from "../../lib/editor/types";

const props = defineProps<{
  style: EditorStyle;
  activeTool: Tool;
  selectedTool: Tool | null;
}>();

const emit = defineEmits<{
  "update:stroke": [value: string];
  "update:strokeWidth": [value: number];
  "update:fontSize": [value: number];
}>();

const { t } = useI18n();

const effectiveTool = computed(
  () => props.selectedTool ?? props.activeTool,
);

const showColor = computed(
  () => effectiveTool.value !== "select" && effectiveTool.value !== "blur",
);

const showThickness = computed(() =>
  ["arrow", "rect", "highlight", "pen"].includes(effectiveTool.value),
);

const showFont = computed(() => effectiveTool.value === "text");

const isVisible = computed(
  () => showColor.value || showThickness.value || showFont.value,
);

function isLightColor(hex: string) {
  const value = hex.replace("#", "");
  if (value.length !== 6) return false;
  const r = Number.parseInt(value.slice(0, 2), 16);
  const g = Number.parseInt(value.slice(2, 4), 16);
  const b = Number.parseInt(value.slice(4, 6), 16);
  return (r * 299 + g * 587 + b * 114) / 1000 > 160;
}
</script>

<template>
  <div
    v-if="isVisible"
    class="flex flex-wrap items-center gap-x-3 gap-y-2 border-b border-sep bg-win/60 px-4 py-2 backdrop-blur-md"
  >
    <template v-if="showColor">
      <span class="text-[11px] font-medium uppercase tracking-wide text-fg-muted">
        {{ t("editor.style.color") }}
      </span>
      <div class="flex items-center gap-1.5">
        <button
          v-for="color in COLOR_PRESETS"
          :key="color"
          type="button"
          class="relative flex size-5 items-center justify-center rounded-full transition-transform duration-150 hover:scale-110"
          :class="
            props.style.stroke === color
              ? 'ring-2 ring-accent ring-offset-2 ring-offset-win'
              : 'shadow-[inset_0_0_0_1px_rgb(0_0_0_/_.2)]'
          "
          :style="{ backgroundColor: color }"
          :aria-label="t('editor.style.colorAria', { color })"
          :aria-pressed="props.style.stroke === color"
          @click="emit('update:stroke', color)"
        >
          <IconCheck
            v-if="props.style.stroke === color"
            class="size-2.5"
            :class="isLightColor(color) ? 'text-black/80' : 'text-white'"
            stroke-width="3"
          />
        </button>
      </div>
    </template>

    <template v-if="showThickness">
      <span
        v-if="showColor"
        class="hidden h-4 w-px bg-sep sm:block"
        aria-hidden="true"
      />
      <span class="text-[11px] font-medium uppercase tracking-wide text-fg-muted">
        {{ t("editor.style.thickness") }}
      </span>
      <div class="inline-flex items-center gap-0.5 rounded-[var(--radius-control)] bg-field p-0.5">
        <button
          v-for="width in STROKE_WIDTHS"
          :key="width"
          type="button"
          class="flex h-7 min-w-8 items-center justify-center rounded-[calc(var(--radius-control)-2px)] px-2 transition-colors duration-150"
          :class="
            props.style.strokeWidth === width
              ? 'bg-elev text-fg shadow-sm'
              : 'text-fg-muted hover:text-fg'
          "
          :aria-pressed="props.style.strokeWidth === width"
          :aria-label="`${t('editor.style.thickness')} ${width}`"
          @click="emit('update:strokeWidth', width)"
        >
          <span
            class="block rounded-full bg-current"
            :style="{
              width: `${6 + width * 2}px`,
              height: `${Math.max(2, width)}px`,
            }"
          />
        </button>
      </div>
    </template>

    <template v-if="showFont">
      <span
        v-if="showColor"
        class="hidden h-4 w-px bg-sep sm:block"
        aria-hidden="true"
      />
      <span class="text-[11px] font-medium uppercase tracking-wide text-fg-muted">
        {{ t("editor.style.font") }}
      </span>
      <div class="inline-flex items-center gap-0.5 rounded-[var(--radius-control)] bg-field p-0.5">
        <button
          v-for="size in FONT_SIZES"
          :key="size"
          type="button"
          class="h-7 min-w-8 rounded-[calc(var(--radius-control)-2px)] px-2 text-xs tabular-nums transition-colors duration-150"
          :class="
            props.style.fontSize === size
              ? 'bg-elev font-medium text-fg shadow-sm'
              : 'text-fg-muted hover:text-fg'
          "
          :aria-pressed="props.style.fontSize === size"
          @click="emit('update:fontSize', size)"
        >
          {{ size }}
        </button>
      </div>
    </template>
  </div>
</template>
