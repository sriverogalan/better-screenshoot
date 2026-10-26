<script setup lang="ts">
import { IconAlertTriangleFilled, IconExclamationCircleFilled, IconInfoCircleFilled } from "@tabler/icons-vue";

withDefaults(
  defineProps<{
    tone?: "warning" | "danger" | "info";
  }>(),
  {
    tone: "warning",
  },
);

const toneClasses: Record<string, string> = {
  warning: "bg-warning-bg shadow-[inset_0_0_0_.5px_var(--color-warning-border)]",
  danger: "bg-danger/10 shadow-[inset_0_0_0_.5px_color-mix(in_srgb,var(--color-danger)_35%,transparent)]",
  info: "bg-accent/10 shadow-[inset_0_0_0_.5px_color-mix(in_srgb,var(--color-accent)_35%,transparent)]",
};

const iconClasses: Record<string, string> = {
  warning: "text-warning",
  danger: "text-danger",
  info: "text-accent",
};

const icons = {
  warning: IconAlertTriangleFilled,
  danger: IconExclamationCircleFilled,
  info: IconInfoCircleFilled,
};
</script>

<template>
  <div
    class="flex gap-2.5 rounded-[var(--radius-group)] px-3 py-2.5 text-body text-fg"
    :class="toneClasses[tone]"
    :role="tone === 'danger' ? 'alert' : 'status'"
  >
    <component
      :is="icons[tone]"
      class="mt-px size-4 shrink-0"
      :class="iconClasses[tone]"
      aria-hidden="true"
    />
    <div class="min-w-0 flex-1">
      <div class="space-y-1">
        <slot />
      </div>
      <div v-if="$slots.actions" class="mt-2.5 flex flex-wrap gap-2">
        <slot name="actions" />
      </div>
    </div>
  </div>
</template>
