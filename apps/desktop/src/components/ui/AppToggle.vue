<script setup lang="ts">
const model = defineModel<boolean>({ required: true });

defineProps<{
  disabled?: boolean;
}>();
</script>

<template>
  <label class="inline-flex cursor-pointer items-center" :class="{ 'opacity-50 cursor-not-allowed': disabled }">
    <input
      v-model="model"
      type="checkbox"
      class="sr-only"
      :disabled="disabled"
    />
    <span
      class="relative inline-block h-[22px] w-[36px] rounded-full transition-colors duration-200"
      :class="model ? 'bg-accent' : 'bg-sep'"
    >
      <span
        class="absolute left-[2px] top-[2px] inline-block h-[18px] w-[18px] rounded-full bg-white shadow-[0_1px_3px_rgb(0_0_0_/_0.3)] transition-transform duration-200"
        :class="model ? 'translate-x-[14px]' : 'translate-x-0'"
      />
    </span>
  </label>
</template>
