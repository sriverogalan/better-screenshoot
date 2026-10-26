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
    class="h-7 w-full rounded-[var(--radius-control)] bg-field px-2.5 text-body text-fg shadow-control outline-none transition-shadow placeholder:text-fg-tertiary focus:shadow-[0_0_0_3px_color-mix(in_srgb,var(--color-accent)_45%,transparent)] disabled:opacity-40"
    :class="mono ? 'font-mono' : undefined"
    @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    @change="$emit('change', ($event.target as HTMLInputElement).value)"
  />
</template>
