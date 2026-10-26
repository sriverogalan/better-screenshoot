<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: "primary" | "secondary" | "danger" | "plain";
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    type?: "button" | "submit";
  }>(),
  {
    variant: "secondary",
    size: "md",
    disabled: false,
    type: "button",
  },
);

defineEmits<{ click: [MouseEvent] }>();

const variantClasses: Record<string, string> = {
  primary:
    "bg-accent text-white shadow-control bg-linear-to-b from-white/12 to-transparent enabled:hover:brightness-110 enabled:active:brightness-90",
  secondary:
    "bg-control text-fg shadow-control enabled:hover:bg-control-hover enabled:active:bg-selected",
  danger:
    "bg-danger text-white shadow-control bg-linear-to-b from-white/12 to-transparent enabled:hover:brightness-110 enabled:active:brightness-90",
  plain: "text-accent enabled:hover:bg-hover enabled:active:bg-selected",
};

const sizeClasses: Record<string, string> = {
  sm: "h-[22px] px-2.5 text-subhead rounded-[5px]",
  md: "h-7 px-3 text-body rounded-[var(--radius-control)]",
  lg: "h-9 min-w-28 px-5 text-body font-medium rounded-lg",
};
</script>

<template>
  <button
    :type="type"
    class="inline-flex shrink-0 select-none items-center justify-center gap-1.5 whitespace-nowrap transition-[background-color,filter] duration-100 disabled:opacity-40"
    :class="[variantClasses[variant], sizeClasses[size]]"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>
