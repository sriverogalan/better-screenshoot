<script setup lang="ts">
withDefaults(
  defineProps<{
    tone?: "warning" | "danger";
  }>(),
  {
    tone: "warning",
  },
);

const toneClasses: Record<string, string> = {
  warning: "border-warning-border bg-warning-bg text-warning-fg",
  danger: "border-danger/40 bg-danger/10 text-danger",
};
</script>

<template>
  <div
    class="rounded-xl border px-4 py-3 text-sm"
    :class="toneClasses[tone]"
    :role="tone === 'danger' ? 'alert' : 'status'"
  >
    <div class="space-y-1">
      <slot />
    </div>
    <div v-if="$slots.actions" class="mt-3 flex flex-wrap gap-2">
      <slot name="actions" />
    </div>
  </div>
</template>
