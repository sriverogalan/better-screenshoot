<script setup lang="ts">
import type { Component } from "vue";

withDefaults(
  defineProps<{
    icon: Component;
    tint: string;
    size?: "sm" | "lg" | "xl";
  }>(),
  {
    size: "sm",
  },
);

const boxClasses: Record<string, string> = {
  sm: "size-5 rounded-[5px]",
  lg: "size-10 rounded-[10px]",
  xl: "size-16 rounded-[15px]",
};

const glyphClasses: Record<string, string> = {
  sm: "size-[13px]",
  lg: "size-6",
  xl: "size-9",
};
</script>

<template>
  <span
    class="inline-flex shrink-0 items-center justify-center text-white shadow-[inset_0_0_0_.5px_rgb(0_0_0_/_.12),0_.5px_1px_rgb(0_0_0_/_.15)]"
    :class="boxClasses[size]"
    :style="{
      backgroundImage: `linear-gradient(180deg, color-mix(in srgb, ${tint} 78%, white), ${tint})`,
    }"
    aria-hidden="true"
  >
    <component :is="icon" :class="glyphClasses[size]" stroke-width="2" />
  </span>
</template>
