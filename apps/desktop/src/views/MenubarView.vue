<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import MenubarActions from "../components/menubar/MenubarActions.vue";
import MenubarRecents from "../components/menubar/MenubarRecents.vue";
import MenubarFooter from "../components/menubar/MenubarFooter.vue";
import { useSettingsStore } from "../stores/settings";

const settingsStore = useSettingsStore();
const panelRef = ref<HTMLElement | null>(null);

const NAV_KEYS = ["ArrowDown", "ArrowUp", "Home", "End"];

async function hideWindow() {
  await getCurrentWindow().hide();
}

function focusableItems(): HTMLElement[] {
  const root = panelRef.value;
  if (!root) return [];
  return Array.from(root.querySelectorAll<HTMLElement>("button:not(:disabled)"));
}

function navigateItems(key: string) {
  const items = focusableItems();
  if (items.length === 0) return;

  const active = document.activeElement as HTMLElement | null;
  const currentIndex = active ? items.indexOf(active) : -1;

  let nextIndex = currentIndex;
  if (key === "ArrowDown") nextIndex = currentIndex + 1 >= items.length ? 0 : currentIndex + 1;
  else if (key === "ArrowUp") nextIndex = currentIndex - 1 < 0 ? items.length - 1 : currentIndex - 1;
  else if (key === "Home") nextIndex = 0;
  else if (key === "End") nextIndex = items.length - 1;

  items[nextIndex]?.focus();
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    void hideWindow();
    return;
  }

  if (NAV_KEYS.includes(e.key)) {
    e.preventDefault();
    navigateItems(e.key);
  }
}

onMounted(async () => {
  document.documentElement.classList.add("vibrancy-mode");
  await settingsStore.load();
  window.addEventListener("blur", hideWindow);
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  document.documentElement.classList.remove("vibrancy-mode");
  window.removeEventListener("blur", hideWindow);
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <div class="flex h-full flex-col">
    <div
      ref="panelRef"
      class="menubar-panel flex flex-1 flex-col overflow-hidden rounded-[13px] bg-win/30"
    >
      <MenubarActions />
      <MenubarRecents />
      <div class="flex-1" />
      <MenubarFooter />
    </div>
  </div>
</template>

<style scoped>
.menubar-panel {
  position: relative;
  transform-origin: top center;
  animation: menubar-in 160ms cubic-bezier(0.2, 0.9, 0.3, 1.1);
}

@keyframes menubar-in {
  from {
    opacity: 0;
    transform: scale(0.97) translateY(-4px);
  }

  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}
</style>
