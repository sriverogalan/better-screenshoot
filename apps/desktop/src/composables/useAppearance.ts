import { watch } from "vue";
import { storeToRefs } from "pinia";
import type { AppAppearance } from "@better-screenshoot/shared-types";
import { useSettingsStore } from "../stores/settings";

const APPEARANCE_CLASSES = ["theme-auto", "theme-light", "theme-dark"] as const;

export function applyAppearance(value: AppAppearance): void {
  const { classList } = document.documentElement;
  APPEARANCE_CLASSES.forEach((cls) => classList.remove(cls));
  classList.add(`theme-${value}`);
}

export function useAppearance() {
  const settingsStore = useSettingsStore();
  const { settings } = storeToRefs(settingsStore);

  watch(
    () => settings.value.appearance,
    (value) => {
      applyAppearance(value);
    },
    { immediate: true },
  );

  return { applyAppearance };
}
