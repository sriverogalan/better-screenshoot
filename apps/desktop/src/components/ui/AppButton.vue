<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "danger";
    disabled?: boolean;
    type?: "button" | "submit";
  }>(),
  {
    variant: "secondary",
    disabled: false,
    type: "button",
  },
);

defineEmits<{ click: [MouseEvent] }>();

const variantClasses: Record<string, string> = {
  primary: "bg-accent text-white hover:bg-accent/90",
  secondary: "border border-sep bg-elev text-fg hover:bg-win",
  danger: "bg-danger text-white hover:bg-danger/90",
};
</script>

<template>
  <button
    :type="type"
    class="inline-flex items-center justify-center gap-1.5 rounded-[var(--radius-control)] px-3 py-2 text-sm font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50"
    :class="variantClasses[variant]"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>
