<script setup lang="ts">
import { nextTick, onUnmounted, ref } from "vue";

const props = withDefaults(
  defineProps<{
    label: string;
    shortcut?: string;
    delayMs?: number;
    disabled?: boolean;
  }>(),
  {
    delayMs: 400,
    disabled: false,
  },
);

const visible = ref(false);
const style = ref({ left: "0px", top: "0px" });
const triggerRef = ref<HTMLElement | null>(null);
let showTimer: ReturnType<typeof setTimeout> | null = null;

function clearTimer() {
  if (showTimer !== null) {
    clearTimeout(showTimer);
    showTimer = null;
  }
}

function place() {
  const el = triggerRef.value;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  const tipWidth = 180;
  const left = Math.max(
    8,
    Math.min(
      rect.left + rect.width / 2 - tipWidth / 2,
      window.innerWidth - tipWidth - 8,
    ),
  );
  const top = rect.bottom + 6;
  style.value = {
    left: `${left}px`,
    top: `${Math.min(top, window.innerHeight - 40)}px`,
  };
}

function onEnter() {
  if (props.disabled || !props.label) return;
  clearTimer();
  showTimer = setTimeout(async () => {
    visible.value = true;
    await nextTick();
    place();
  }, props.delayMs);
}

function onLeave() {
  clearTimer();
  visible.value = false;
}

onUnmounted(clearTimer);
</script>

<template>
  <span
    ref="triggerRef"
    class="inline-flex"
    @mouseenter="onEnter"
    @mouseleave="onLeave"
    @focusin="onEnter"
    @focusout="onLeave"
  >
    <slot />
  </span>
  <Teleport to="body">
    <div
      v-if="visible"
      class="app-tooltip pointer-events-none fixed z-[100] max-w-[220px] rounded-[7px] bg-material px-2.5 py-1.5 text-caption text-fg shadow-popover backdrop-blur-2xl backdrop-saturate-150"
      :style="style"
      role="tooltip"
    >
      <span class="font-medium">{{ label }}</span>
      <span
        v-if="shortcut"
        class="ml-1.5 tabular-nums text-fg-muted"
      >{{ shortcut }}</span>
    </div>
  </Teleport>
</template>

<style scoped>
.app-tooltip {
  animation: tip-in 100ms ease-out;
}

@keyframes tip-in {
  from {
    opacity: 0;
    transform: translateY(-2px);
  }
}
</style>
