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
import AppTooltip from "../ui/AppTooltip.vue";

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
    class="relative z-[5] flex h-11 shrink-0 items-center justify-center gap-4 overflow-x-auto bg-win/95 px-4 shadow-[0_.5px_0_var(--color-sep)]"
  >
    <div v-if="showColor" class="flex items-center gap-2.5">
      <span class="text-caption font-medium text-fg-muted">
        {{ t("editor.style.color") }}
      </span>
      <div class="flex items-center gap-1.5">
        <AppTooltip
          v-for="color in COLOR_PRESETS"
          :key="color"
          :label="t('editor.style.colorAria', { color })"
        >
          <button
            type="button"
            class="relative flex size-[22px] items-center justify-center rounded-full transition-transform duration-100 active:scale-90"
            :class="
              props.style.stroke === color
                ? 'shadow-[0_0_0_2px_var(--color-win),0_0_0_4px_var(--color-fg-muted)]'
                : 'shadow-[inset_0_0_0_.5px_rgb(0_0_0_/_.25)] hover:scale-110'
            "
            :style="{ backgroundColor: color }"
            :aria-label="t('editor.style.colorAria', { color })"
            :aria-pressed="props.style.stroke === color"
            @click="emit('update:stroke', color)"
          >
            <IconCheck
              v-if="props.style.stroke === color"
              class="size-3"
              :class="isLightColor(color) ? 'text-black/75' : 'text-white'"
              stroke-width="3"
            />
          </button>
        </AppTooltip>
      </div>
    </div>

    <span
      v-if="showColor && (showThickness || showFont)"
      class="h-5 w-px shrink-0 bg-sep"
      aria-hidden="true"
    />

    <div v-if="showThickness" class="flex items-center gap-2.5">
      <span class="text-caption font-medium text-fg-muted">
        {{ t("editor.style.thickness") }}
      </span>
      <div class="inline-flex items-center gap-px rounded-[7px] bg-hover p-[2px] shadow-[inset_0_0_0_.5px_var(--color-sep)]">
        <AppTooltip
          v-for="width in STROKE_WIDTHS"
          :key="width"
          :label="`${t('editor.style.thickness')} ${width}`"
        >
          <button
            type="button"
            class="flex h-6 w-9 items-center justify-center rounded-[5px] transition-colors duration-100"
            :class="
              props.style.strokeWidth === width
                ? 'bg-segment text-fg shadow-control'
                : 'text-fg-muted hover:text-fg'
            "
            :aria-pressed="props.style.strokeWidth === width"
            :aria-label="`${t('editor.style.thickness')} ${width}`"
            @click="emit('update:strokeWidth', width)"
          >
            <span
              class="block rounded-full bg-current"
              :style="{
                width: '18px',
                height: `${Math.max(1.5, width * 0.9)}px`,
              }"
            />
          </button>
        </AppTooltip>
      </div>
    </div>

    <div v-if="showFont" class="flex items-center gap-2.5">
      <span class="text-caption font-medium text-fg-muted">
        {{ t("editor.style.font") }}
      </span>
      <div class="inline-flex items-center gap-px rounded-[7px] bg-hover p-[2px] shadow-[inset_0_0_0_.5px_var(--color-sep)]">
        <AppTooltip
          v-for="size in FONT_SIZES"
          :key="size"
          :label="`${t('editor.style.font')} ${size}`"
        >
          <button
            type="button"
            class="h-6 min-w-9 rounded-[5px] px-2 text-subhead tabular-nums transition-colors duration-100"
            :class="
              props.style.fontSize === size
                ? 'bg-segment font-medium text-fg shadow-control'
                : 'text-fg-muted hover:text-fg'
            "
            :aria-pressed="props.style.fontSize === size"
            :aria-label="`${t('editor.style.font')} ${size}`"
            @click="emit('update:fontSize', size)"
          >
            {{ size }}
          </button>
        </AppTooltip>
      </div>
    </div>
  </div>
</template>
