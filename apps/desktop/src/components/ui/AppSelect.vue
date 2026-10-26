<script setup lang="ts">
import { IconSelector } from "@tabler/icons-vue";

export interface AppSelectOption {
  value: string;
  label: string;
}

defineProps<{
  modelValue: string;
  options: AppSelectOption[];
  disabled?: boolean;
  id?: string;
  ariaLabel?: string;
}>();

defineEmits<{
  "update:modelValue": [value: string];
}>();
</script>

<template>
  <span class="relative inline-flex shrink-0" :class="disabled ? 'opacity-40' : undefined">
    <select
      :id="id"
      :value="modelValue"
      :aria-label="ariaLabel"
      class="h-[24px] appearance-none rounded-[var(--radius-control)] bg-control py-0 pl-2.5 pr-7 text-body text-fg shadow-control outline-none transition-colors hover:bg-control-hover"
      :disabled="disabled"
      @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <option v-for="option in options" :key="option.value" :value="option.value">
        {{ option.label }}
      </option>
    </select>
    <IconSelector
      class="pointer-events-none absolute right-1.5 top-1/2 size-3.5 -translate-y-1/2 text-fg-muted"
      stroke-width="2.25"
      aria-hidden="true"
    />
  </span>
</template>
