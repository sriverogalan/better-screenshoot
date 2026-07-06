<script setup lang="ts">
const model = defineModel<string>({ required: true });

defineProps<{
  options: Array<{ value: string; label: string }>;
}>();
</script>

<template>
  <div class="inline-flex rounded-[var(--radius-control)] bg-field p-0.5 gap-0.5">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      class="rounded-[var(--radius-control)] px-3 py-1 text-sm transition-colors duration-150"
      :class="
        model === option.value
          ? 'bg-accent text-white font-medium'
          : 'text-fg-muted hover:text-fg'
      "
      :aria-pressed="model === option.value"
      @click="model = option.value"
    >
      {{ option.label }}
    </button>
  </div>
</template>
