<script setup lang="ts">
withDefaults(
  defineProps<{
    layout?: "row" | "block";
  }>(),
  {
    layout: "row",
  },
);
</script>

<template>
  <div :class="layout === 'row' ? 'flex items-center justify-between gap-4 px-4 py-3' : 'px-4 py-3'">
    <slot />
  </div>
</template>
