<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    disabled?: boolean;
    readonly?: boolean;
    mono?: boolean;
    id?: string;
    type?: "text" | "password" | "search";
  }>(),
  {
    placeholder: "",
    disabled: false,
    readonly: false,
    mono: false,
    type: "text",
  },
);

defineEmits<{
  "update:modelValue": [value: string];
  change: [value: string];
}>();
</script>

<template>
  <input
    :id="id"
    :type="type"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :readonly="readonly"
    class="w-full rounded-[var(--radius-control)] border border-sep bg-field px-3 py-2 text-sm text-fg outline-none transition-colors placeholder:text-fg-muted focus-visible:border-accent disabled:cursor-not-allowed disabled:opacity-60"
    :class="mono ? 'font-mono' : undefined"
    @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    @change="$emit('change', ($event.target as HTMLInputElement).value)"
  />
</template>
