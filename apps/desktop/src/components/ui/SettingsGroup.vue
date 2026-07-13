<script setup lang="ts">
defineProps<{
  label?: string;
}>();
</script>

<template>
  <div class="mt-6 first:mt-0">
    <p
      v-if="label"
      class="px-1 pb-1.5 text-xs font-semibold uppercase tracking-wider text-fg-muted"
    >
      {{ label }}
    </p>
    <div class="divide-y divide-sep overflow-hidden rounded-xl border border-sep bg-elev">
      <slot />
    </div>
  </div>
</template>
