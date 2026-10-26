<script setup lang="ts">
const model = defineModel<boolean>({ required: true });

defineProps<{
  disabled?: boolean;
  label?: string;
}>();
</script>

<template>
  <label
    class="inline-flex shrink-0 items-center"
    :class="disabled ? 'opacity-40' : undefined"
  >
    <input
      v-model="model"
      type="checkbox"
      role="switch"
      class="peer sr-only"
      :aria-label="label"
      :disabled="disabled"
    />
    <span
      class="relative inline-block h-[20px] w-[34px] rounded-full shadow-[inset_0_0_0_.5px_rgb(0_0_0_/_.12)] transition-colors duration-200 ease-out peer-focus-visible:outline-3 peer-focus-visible:outline-accent/50 peer-focus-visible:outline-offset-1"
      :class="model ? 'bg-accent' : 'bg-fg-tertiary'"
    >
      <span
        class="absolute left-[2px] top-[2px] inline-block size-4 rounded-full bg-white shadow-[0_1px_2px_rgb(0_0_0_/_.3),0_0_0_.5px_rgb(0_0_0_/_.06)] transition-transform duration-200 ease-[cubic-bezier(.3,1.4,.6,1)]"
        :class="model ? 'translate-x-[14px]' : 'translate-x-0'"
      />
    </span>
  </label>
</template>
