<script setup lang="ts">
withDefaults(
  defineProps<{
    tone?: "warning" | "success" | "neutral";
  }>(),
  {
    tone: "neutral",
  },
);

const toneClasses: Record<string, string> = {
  warning: "bg-warning-bg text-warning-fg",
  success: "bg-success-bg text-success-fg",
  neutral: "bg-hover text-fg-muted",
};

const dotClasses: Record<string, string> = {
  warning: "bg-warning",
  success: "bg-success",
  neutral: "bg-fg-tertiary",
};
</script>

<template>
  <span
    class="inline-flex h-5 shrink-0 items-center gap-1.5 rounded-full px-2 text-caption font-medium"
    :class="toneClasses[tone]"
  >
    <span class="size-1.5 rounded-full" :class="dotClasses[tone]" aria-hidden="true" />
    <slot />
  </span>
</template>
