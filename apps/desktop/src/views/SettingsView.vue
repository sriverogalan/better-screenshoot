<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute, useRouter } from "vue-router";
import type { AppAppearance, AppLocale } from "@better-screenshoot/shared-types";
import { useSettingsStore } from "../stores/settings";
import { useAppearance } from "../composables/useAppearance";
import AppToggle from "../components/ui/AppToggle.vue";
import AppSegmentedControl from "../components/ui/AppSegmentedControl.vue";
import AppButton from "../components/ui/AppButton.vue";
import AppSelect from "../components/ui/AppSelect.vue";
import SettingsGroup from "../components/ui/SettingsGroup.vue";
import SettingsRow from "../components/ui/SettingsRow.vue";
import CaptureSettingsSection from "../components/settings/CaptureSettingsSection.vue";
import CaptureModeSettingsSection from "../components/settings/CaptureModeSettingsSection.vue";
import HotkeysSettingsSection from "../components/settings/HotkeysSettingsSection.vue";
import AppUpdateSection from "../components/settings/AppUpdateSection.vue";
import PendingCaptureBanner from "../components/PendingCaptureBanner.vue";
import { SUPPORTED_LOCALES, setLocale } from "../i18n";

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const settingsStore = useSettingsStore();
const { applyAppearance } = useAppearance();

const settings = computed(() => settingsStore.settings);
const isReplaceMode = computed(() => settings.value.system_capture_mode === "replace_system");

const languageOptions = computed(() =>
  SUPPORTED_LOCALES.map((locale) => ({
    value: locale,
    label: t(`settings.languageOptions.${locale}`),
  })),
);

const appearanceOptions = computed(() => [
  { value: "auto", label: t("settings.appearanceOptions.auto") },
  { value: "light", label: t("settings.appearanceOptions.light") },
  { value: "dark", label: t("settings.appearanceOptions.dark") },
]);

const isOnOnboardingRoute = computed(() => route.path === "/onboarding");

async function updateField<K extends keyof typeof settings.value>(
  key: K,
  value: (typeof settings.value)[K],
) {
  await settingsStore.save({ ...settings.value, [key]: value });
}

async function updateLocale(locale: string) {
  const next = locale as AppLocale;
  await settingsStore.save({ ...settings.value, locale: next });
  await setLocale(next);
}

async function onAppearanceChange(value: string) {
  const appearance = value as AppAppearance;
  await settingsStore.save({ ...settings.value, appearance });
  applyAppearance(appearance);
}

async function runSetupWizardAgain() {
  await settingsStore.save({ ...settings.value, onboarding_completed: false });
  await router.push("/onboarding");
}
</script>

<template>
  <div class="flex min-h-full flex-col bg-win">
    <div data-tauri-drag-region class="h-8 shrink-0" />

    <h1 class="px-6 pb-2 pt-4 text-lg font-semibold text-fg">
      {{ t("settings.title") }}
    </h1>

    <main class="flex-1 overflow-y-auto pb-8">
      <div class="mx-auto w-full max-w-2xl px-6">
        <PendingCaptureBanner />

        <SettingsGroup :label="t('settings.language')">
          <SettingsRow>
            <AppSelect
              class="ml-auto"
              :model-value="settings.locale"
              :options="languageOptions"
              @update:model-value="updateLocale"
            />
          </SettingsRow>
        </SettingsGroup>

        <CaptureSettingsSection />

        <SettingsGroup :label="t('settings.sections.appearance')">
          <SettingsRow>
            <span class="text-sm">{{ t("settings.theme") }}</span>
            <AppSegmentedControl
              :model-value="settings.appearance"
              :options="appearanceOptions"
              @update:model-value="onAppearanceChange"
            />
          </SettingsRow>
        </SettingsGroup>

        <SettingsGroup :label="t('settings.sections.integrations')">
          <SettingsRow>
            <span class="text-sm">{{ t("settings.allowExternalControl") }}</span>
            <AppToggle
              :model-value="settings.allow_external_control"
              @update:model-value="updateField('allow_external_control', $event)"
            />
          </SettingsRow>
          <SettingsRow layout="block">
            <p class="text-xs text-fg-muted">
              {{
                t("settings.urlSchemeHint", {
                  scheme: "betterscreenshoot://capture-area",
                })
              }}
            </p>
          </SettingsRow>
        </SettingsGroup>

        <CaptureModeSettingsSection />

        <HotkeysSettingsSection :is-replace-mode="isReplaceMode" />

        <AppUpdateSection />

        <SettingsGroup :label="t('settings.sections.setup')">
          <SettingsRow>
            <AppButton
              variant="secondary"
              :disabled="isOnOnboardingRoute"
              @click="runSetupWizardAgain"
            >
              {{ t("settings.runSetupWizard") }}
            </AppButton>
          </SettingsRow>
        </SettingsGroup>
      </div>
    </main>
  </div>
</template>
