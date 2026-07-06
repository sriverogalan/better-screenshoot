<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import MenubarHeader from "../components/menubar/MenubarHeader.vue";
import MenubarActions from "../components/menubar/MenubarActions.vue";
import MenubarRecents from "../components/menubar/MenubarRecents.vue";
import MenubarFooter from "../components/menubar/MenubarFooter.vue";
import { useSettingsStore } from "../stores/settings";

const settingsStore = useSettingsStore();

async function hideWindow() {
  await getCurrentWindow().hide();
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    void hideWindow();
  }
}

onMounted(async () => {
  await settingsStore.load();
  window.addEventListener("blur", hideWindow);
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("blur", hideWindow);
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <div
    class="flex h-full flex-col overflow-hidden rounded-[var(--radius-window)] bg-win shadow-window"
    style="box-shadow: var(--shadow-window);"
  >
    <MenubarHeader />
    <MenubarActions />
    <MenubarRecents />
    <MenubarFooter />
  </div>
</template>
