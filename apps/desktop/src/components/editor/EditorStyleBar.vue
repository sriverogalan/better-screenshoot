<script setup lang="ts">
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
</script>

<template>
  <div class="flex flex-wrap items-center gap-2 border-t border-border/60 px-4 py-2">
    <span class="text-xs text-text-muted">Color</span>
    <div class="flex items-center gap-1">
      <button
        v-for="color in COLOR_PRESETS"
        :key="color"
        type="button"
        class="size-6 rounded-full border-2 transition hover:scale-110"
        :class="
          props.style.stroke === color
            ? 'border-white'
            : 'border-transparent'
        "
        :style="{ backgroundColor: color }"
        :aria-label="`Color ${color}`"
        :aria-pressed="props.style.stroke === color"
        @click="emit('update:stroke', color)"
      />
    </div>

    <span class="mx-1 h-5 w-px bg-border" />

    <span class="text-xs text-text-muted">Grosor</span>
    <div class="flex items-center gap-1">
      <button
        v-for="width in STROKE_WIDTHS"
        :key="width"
        type="button"
        class="rounded-lg px-2 py-1 text-xs hover:bg-surface-raised"
        :class="{
          'bg-surface-raised text-accent': props.style.strokeWidth === width,
        }"
        :aria-pressed="props.style.strokeWidth === width"
        @click="emit('update:strokeWidth', width)"
      >
        {{ width }}px
      </button>
    </div>

    <span class="mx-1 h-5 w-px bg-border" />

    <span class="text-xs text-text-muted">Fuente</span>
    <div class="flex items-center gap-1">
      <button
        v-for="size in FONT_SIZES"
        :key="size"
        type="button"
        class="rounded-lg px-2 py-1 text-xs hover:bg-surface-raised"
        :class="{
          'bg-surface-raised text-accent': props.style.fontSize === size,
        }"
        :aria-pressed="props.style.fontSize === size"
        @click="emit('update:fontSize', size)"
      >
        {{ size }}
      </button>
    </div>
  </div>
</template>
