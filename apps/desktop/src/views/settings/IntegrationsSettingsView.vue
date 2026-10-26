<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "../../stores/settings";
import SettingsPage from "../../components/settings/SettingsPage.vue";
import SettingsGroup from "../../components/ui/SettingsGroup.vue";
import SettingsRow from "../../components/ui/SettingsRow.vue";
import AppToggle from "../../components/ui/AppToggle.vue";

const URL_SCHEME = "betterscreenshoot://capture-area";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const settings = computed(() => settingsStore.settings);

async function updateExternalControl(value: boolean) {
  await settingsStore.save({ ...settings.value, allow_external_control: value });
}
</script>

<template>
  <SettingsPage>
    <SettingsGroup :label="t('settings.sections.integrations')">
      <SettingsRow :label="t('settings.allowExternalControl')">
        <AppToggle
          :model-value="settings.allow_external_control"
          :label="t('settings.allowExternalControl')"
          @update:model-value="updateExternalControl"
        />
      </SettingsRow>
      <SettingsRow :label="t('settings.urlScheme')">
        <code
          class="select-text truncate rounded-[5px] bg-hover px-2 py-0.5 font-mono text-subhead text-fg-muted"
        >
          {{ URL_SCHEME }}
        </code>
      </SettingsRow>
    </SettingsGroup>
  </SettingsPage>
</template>
