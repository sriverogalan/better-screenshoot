<script setup lang="ts">
import { computed } from "vue";
import { useSettingsStore } from "../../stores/settings";
import SettingsPage from "../../components/settings/SettingsPage.vue";
import HotkeysSettingsSection from "../../components/settings/HotkeysSettingsSection.vue";

const settingsStore = useSettingsStore();
const isReplaceMode = computed(
  () => settingsStore.settings.system_capture_mode === "replace_system",
);
</script>

<template>
  <SettingsPage>
    <HotkeysSettingsSection :is-replace-mode="isReplaceMode" />
  </SettingsPage>
</template>
