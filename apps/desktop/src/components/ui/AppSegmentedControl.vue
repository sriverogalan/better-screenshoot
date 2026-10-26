<script setup lang="ts">
const model = defineModel<string>({ required: true });

defineProps<{
  options: Array<{ value: string; label: string }>;
}>();
</script>

<template>
  <div class="inline-flex shrink-0 gap-px rounded-[7px] bg-hover p-[2px] shadow-[inset_0_0_0_.5px_var(--color-sep)]">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      class="h-[20px] min-w-16 rounded-[5px] px-3 text-subhead transition-[background-color,color,box-shadow] duration-150"
      :class="
        model === option.value
          ? 'bg-segment font-medium text-fg shadow-control'
          : 'text-fg-muted hover:text-fg'
      "
      :aria-pressed="model === option.value"
      @click="model = option.value"
    >
      {{ option.label }}
    </button>
  </div>
</template>
