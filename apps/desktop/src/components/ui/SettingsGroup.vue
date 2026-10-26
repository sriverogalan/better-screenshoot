<script setup lang="ts">
defineProps<{
  label?: string;
  footer?: string;
}>();
</script>

<template>
  <section class="mt-6 first:mt-0">
    <h2 v-if="label" class="px-2.5 pb-2 text-body font-semibold text-fg">
      {{ label }}
    </h2>
    <div class="overflow-hidden rounded-[var(--radius-group)] bg-group shadow-[inset_0_0_0_.5px_var(--color-sep)]">
      <slot />
    </div>
    <p v-if="footer || $slots.footer" class="px-2.5 pt-2 text-subhead text-fg-muted">
      <slot name="footer">{{ footer }}</slot>
    </p>
  </section>
</template>
