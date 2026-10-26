<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import type { AppAppearance } from "@better-screenshoot/shared-types";

const model = defineModel<AppAppearance>({ required: true });

const { t } = useI18n();

const options = computed(() => [
  { value: "auto" as const, label: t("settings.appearanceOptions.auto") },
  { value: "light" as const, label: t("settings.appearanceOptions.light") },
  { value: "dark" as const, label: t("settings.appearanceOptions.dark") },
]);
</script>

<template>
  <div class="flex gap-4" role="radiogroup" :aria-label="t('settings.theme')">
    <button
      v-for="option in options"
      :key="option.value"
      type="button"
      role="radio"
      class="group flex flex-col items-center gap-1.5 outline-none"
      :aria-checked="model === option.value"
      @click="model = option.value"
    >
      <span
        class="relative block h-[46px] w-[68px] overflow-hidden rounded-[7px] transition-shadow duration-150"
        :class="
          model === option.value
            ? 'shadow-[0_0_0_3px_var(--color-accent)]'
            : 'shadow-[0_0_0_.5px_var(--color-sep),0_1px_2px_rgb(0_0_0_/_.15)] group-hover:shadow-[0_0_0_.5px_var(--color-sep),0_0_0_3px_var(--color-hover)]'
        "
      >
        <span class="thumb absolute inset-0" :class="`thumb-${option.value}`" />
        <span class="thumb-window absolute bottom-0 left-2.5 right-0 top-2 rounded-tl-[4px]" :class="`thumb-${option.value}`">
          <span class="flex gap-[2px] p-[3px]">
            <span class="size-[3px] rounded-full bg-[#FF5F57]" />
            <span class="size-[3px] rounded-full bg-[#FEBC2E]" />
            <span class="size-[3px] rounded-full bg-[#28C840]" />
          </span>
          <span class="thumb-accent ml-[3px] mt-[2px] block h-[4px] w-5 rounded-full" />
          <span class="thumb-line ml-[3px] mt-[3px] block h-[3px] w-8 rounded-full" />
          <span class="thumb-line ml-[3px] mt-[2px] block h-[3px] w-6 rounded-full" />
        </span>
      </span>
      <span
        class="text-subhead"
        :class="model === option.value ? 'font-semibold text-fg' : 'text-fg-muted'"
      >
        {{ option.label }}
      </span>
    </button>
  </div>
</template>

<style scoped>
.thumb-light.thumb {
  background: linear-gradient(135deg, #7ab8ff, #c79bff);
}

.thumb-dark.thumb {
  background: linear-gradient(135deg, #1b3a6b, #3d1f5c);
}

.thumb-auto.thumb {
  background: linear-gradient(90deg, #7ab8ff 0 50%, #1b3a6b 50% 100%);
}

.thumb-light.thumb-window {
  background: #f5f5f7;
  --thumb-line: rgb(0 0 0 / 0.14);
}

.thumb-dark.thumb-window {
  background: #2c2c2e;
  --thumb-line: rgb(255 255 255 / 0.2);
}

.thumb-auto.thumb-window {
  background: linear-gradient(90deg, #f5f5f7 0 calc(50% - 5px), #2c2c2e calc(50% - 5px) 100%);
  --thumb-line: rgb(128 128 128 / 0.35);
}

.thumb-accent {
  background: #0a84ff;
}

.thumb-line {
  background: var(--thumb-line);
}
</style>
