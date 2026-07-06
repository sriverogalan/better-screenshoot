<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { EditorStyle } from "../../lib/editor/types";
import {
  COLOR_PRESETS,
  FONT_SIZES,
  STROKE_WIDTHS,
} from "../../lib/editor/types";

const props = defineProps<{
  style: EditorStyle;
}>();

const emit = defineEmits<{
  "update:stroke": [value: string];
  "update:strokeWidth": [value: number];
  "update:fontSize": [value: number];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="flex flex-wrap items-center gap-2 border-t border-sep/60 px-4 py-2">
    <span class="text-xs text-fg-muted">{{ t("editor.style.color") }}</span>
    <div class="flex items-center gap-1">
      <button
        v-for="color in COLOR_PRESETS"
        :key="color"
        type="button"
        class="size-5 rounded-full transition hover:scale-110"
        :class="
          props.style.stroke === color
            ? 'ring-2 ring-white ring-offset-1 ring-offset-win'
            : ''
        "
        :style="{ backgroundColor: color }"
        :aria-label="t('editor.style.colorAria', { color })"
        :aria-pressed="props.style.stroke === color"
        @click="emit('update:stroke', color)"
      />
    </div>

    <span class="mx-1 h-5 w-px bg-sep" />

    <span class="text-xs text-fg-muted">{{ t("editor.style.thickness") }}</span>
    <div class="flex items-center gap-1">
      <button
        v-for="width in STROKE_WIDTHS"
        :key="width"
        type="button"
        class="rounded-lg px-2 py-0.5 text-xs transition"
        :class="
          props.style.strokeWidth === width
            ? 'bg-accent text-white'
            : 'text-fg-muted hover:bg-elev hover:text-fg'
        "
        :aria-pressed="props.style.strokeWidth === width"
        @click="emit('update:strokeWidth', width)"
      >
        {{ width }}
      </button>
    </div>

    <span class="mx-1 h-5 w-px bg-sep" />

    <span class="text-xs text-fg-muted">{{ t("editor.style.font") }}</span>
    <div class="flex items-center gap-1">
      <button
        v-for="size in FONT_SIZES"
        :key="size"
        type="button"
        class="rounded-lg px-2 py-0.5 text-xs transition"
        :class="
          props.style.fontSize === size
            ? 'bg-accent text-white'
            : 'text-fg-muted hover:bg-elev hover:text-fg'
        "
        :aria-pressed="props.style.fontSize === size"
        @click="emit('update:fontSize', size)"
      >
        {{ size }}
      </button>
    </div>
  </div>
</template>
