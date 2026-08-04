import { defineStore } from "pinia";
import { ref } from "vue";
import type { AppSettings } from "@better-screenshoot/shared-types";
import { DEFAULT_SETTINGS } from "@better-screenshoot/shared-types";
import { getSettings, updateSettings } from "../lib/tauri";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });
  const loading = ref(false);

  async function load() {
    loading.value = true;
    try {
      settings.value = await getSettings();
    } catch (error) {
      console.error("[settings] Failed to load settings, retaining defaults:", error);
    } finally {
      loading.value = false;
    }
  }

  async function save(next: AppSettings) {
    settings.value = await updateSettings(next);
  }

  return { settings, loading, load, save };
});
