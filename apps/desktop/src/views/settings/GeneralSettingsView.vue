<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import type { AppAppearance, AppLocale } from "@better-screenshoot/shared-types";
import { useSettingsStore } from "../../stores/settings";
import { useAppearance } from "../../composables/useAppearance";
import { SUPPORTED_LOCALES, setLocale } from "../../i18n";
import SettingsPage from "../../components/settings/SettingsPage.vue";
import AppearancePicker from "../../components/settings/AppearancePicker.vue";
import SettingsGroup from "../../components/ui/SettingsGroup.vue";
import SettingsRow from "../../components/ui/SettingsRow.vue";
import AppSelect from "../../components/ui/AppSelect.vue";
import AppButton from "../../components/ui/AppButton.vue";

const { t } = useI18n();
const router = useRouter();
const settingsStore = useSettingsStore();
const { applyAppearance } = useAppearance();

const settings = computed(() => settingsStore.settings);

const languageOptions = computed(() =>
  SUPPORTED_LOCALES.map((locale) => ({
    value: locale,
    label: t(`settings.languageOptions.${locale}`),
  })),
);

async function updateLocale(locale: string) {
  const next = locale as AppLocale;
  await settingsStore.save({ ...settings.value, locale: next });
  await setLocale(next);
}

async function onAppearanceChange(appearance: AppAppearance) {
  await settingsStore.save({ ...settings.value, appearance });
  applyAppearance(appearance);
}

async function runSetupWizardAgain() {
  await settingsStore.save({ ...settings.value, onboarding_completed: false });
  await router.push("/onboarding");
}
</script>

<template>
  <SettingsPage>
    <SettingsGroup :label="t('settings.sections.appearance')">
      <SettingsRow :label="t('settings.theme')">
        <AppearancePicker
          :model-value="settings.appearance"
          @update:model-value="onAppearanceChange"
        />
      </SettingsRow>
      <SettingsRow :label="t('settings.language')">
        <AppSelect
          :model-value="settings.locale"
          :options="languageOptions"
          :aria-label="t('settings.language')"
          @update:model-value="updateLocale"
        />
      </SettingsRow>
    </SettingsGroup>

    <SettingsGroup :label="t('settings.sections.setup')">
      <SettingsRow
        :label="t('settings.runSetupWizard')"
        :description="t('settings.runSetupWizardHint')"
      >
        <AppButton @click="runSetupWizardAgain">
          {{ t("settings.openSetupWizard") }}
        </AppButton>
      </SettingsRow>
    </SettingsGroup>
  </SettingsPage>
</template>
