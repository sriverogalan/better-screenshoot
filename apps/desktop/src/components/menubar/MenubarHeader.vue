<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";

function formatTime(): string {
  return new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

const currentTime = ref(formatTime());
let intervalId: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  intervalId = setInterval(() => {
    currentTime.value = formatTime();
  }, 60_000);
});

onUnmounted(() => {
  if (intervalId !== null) {
    clearInterval(intervalId);
  }
});
</script>

<template>
  <div class="flex justify-end border-b border-sep px-3 py-2">
    <span class="text-xs text-fg-muted">{{ currentTime }}</span>
  </div>
</template>
