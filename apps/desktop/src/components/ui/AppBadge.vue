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
  neutral: "bg-win text-fg-muted border border-sep",
};
</script>

<template>
  <span
    class="inline-flex items-center rounded-md px-2 py-0.5 text-xs font-medium"
    :class="toneClasses[tone]"
  >
    <slot />
  </span>
</template>
