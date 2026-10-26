<script setup lang="ts">
import { ref } from "vue";

defineProps<{
  title: string;
  subtitle?: string;
}>();

const scrolled = ref(false);

function onScroll(event: Event) {
  scrolled.value = (event.target as HTMLElement).scrollTop > 2;
}
</script>

<template>
  <div class="flex h-full min-h-0 flex-col">
    <header
      data-tauri-drag-region
      class="relative z-10 flex h-[52px] shrink-0 items-center gap-3 px-5 transition-shadow duration-200"
      :class="scrolled ? 'shadow-[0_.5px_0_var(--color-sep)]' : undefined"
    >
      <div data-tauri-drag-region class="flex min-w-0 items-baseline gap-2">
        <h1 data-tauri-drag-region class="truncate text-title3 font-bold text-fg">
          {{ title }}
        </h1>
        <span
          v-if="subtitle"
          data-tauri-drag-region
          class="truncate text-subhead tabular-nums text-fg-muted"
        >
          {{ subtitle }}
        </span>
      </div>
      <div v-if="$slots.actions" class="ml-auto flex shrink-0 items-center gap-2">
        <slot name="actions" />
      </div>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto" @scroll.passive="onScroll">
      <slot />
    </div>
  </div>
</template>
