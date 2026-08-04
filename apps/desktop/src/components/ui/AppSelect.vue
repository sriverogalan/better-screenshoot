<script setup lang="ts">
export interface AppSelectOption {
  value: string;
  label: string;
}

defineProps<{
  modelValue: string;
  options: AppSelectOption[];
  disabled?: boolean;
  id?: string;
}>();

defineEmits<{
  "update:modelValue": [value: string];
}>();
</script>

<template>
  <select
    :id="id"
    :value="modelValue"
    class="rounded-[var(--radius-control)] border border-sep bg-field px-3 py-1.5 text-sm text-fg outline-none transition-colors focus-visible:border-accent disabled:cursor-not-allowed disabled:opacity-50"
    :disabled="disabled"
    @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
  >
    <option v-for="option in options" :key="option.value" :value="option.value">
      {{ option.label }}
    </option>
  </select>
</template>
