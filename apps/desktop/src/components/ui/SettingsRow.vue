<script setup lang="ts">
withDefaults(
  defineProps<{
    layout?: "row" | "block";
    label?: string;
    description?: string;
  }>(),
  {
    layout: "row",
    label: undefined,
    description: undefined,
  },
);
</script>

<template>
  <div
    class="relative min-h-10 px-3 py-2 before:absolute before:inset-x-3 before:top-0 before:h-px before:bg-sep first:before:hidden"
    :class="layout === 'row' ? 'flex items-center justify-between gap-4' : undefined"
  >
    <div v-if="label" class="min-w-0 flex-1">
      <p class="text-body text-fg">{{ label }}</p>
      <p v-if="description" class="mt-0.5 text-subhead text-fg-muted">{{ description }}</p>
    </div>
    <slot />
  </div>
</template>
