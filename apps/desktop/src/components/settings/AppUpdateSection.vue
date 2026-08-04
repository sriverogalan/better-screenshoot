<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { IconRefresh } from "@tabler/icons-vue";
import { useAppUpdater } from "../../composables/useAppUpdater";
import AppButton from "../ui/AppButton.vue";
import SettingsGroup from "../ui/SettingsGroup.vue";
import SettingsRow from "../ui/SettingsRow.vue";

const { t } = useI18n();

const {
  phase,
  currentVersion,
  updateSummary,
  errorMessage,
  statusCode,
  downloadedBytes,
  totalBytes,
  updateAvailable,
  loadCurrentVersion,
  checkForUpdates,
  installAvailableUpdate,
} = useAppUpdater();

const isBusy = computed(
  () => phase.value === "checking" || phase.value === "downloading",
);

const statusMessage = computed(() => {
  if (phase.value === "checking") {
    return t("settings.updates.status.checking");
  }

  if (phase.value === "downloading") {
    if (totalBytes.value) {
      const percent = Math.min(
        100,
        Math.round((downloadedBytes.value / totalBytes.value) * 100),
      );
      return t("settings.updates.status.downloadingWithPercent", { percent });
    }

    return t("settings.updates.status.downloading");
  }

  if (statusCode.value) {
    return t(`settings.updates.status.${statusCode.value}`);
  }

  if (updateAvailable.value && updateSummary.value) {
    return t("settings.updates.status.available", {
      version: updateSummary.value.version,
    });
  }

  return errorMessage.value;
});

onMounted(() => {
  void loadCurrentVersion();
});
</script>

<template>
  <SettingsGroup :label="t('settings.updates.title')">
    <SettingsRow>
      <div class="min-w-0 flex-1">
        <p class="text-sm">
          {{ t("settings.updates.currentVersion") }}
          <span class="font-medium text-accent">{{ currentVersion ?? "…" }}</span>
        </p>
        <p class="mt-1 text-xs text-fg-muted">
          {{ t("settings.updates.description") }}
        </p>
      </div>
    </SettingsRow>
    <SettingsRow layout="block">
      <div class="flex flex-wrap gap-2">
        <AppButton variant="secondary" :disabled="isBusy" @click="checkForUpdates()">
          <IconRefresh class="size-4" aria-hidden="true" />
          {{ t("settings.updates.checkForUpdates") }}
        </AppButton>
        <AppButton
          v-if="updateAvailable"
          variant="primary"
          :disabled="isBusy"
          @click="installAvailableUpdate"
        >
          {{ t("settings.updates.installVersion", { version: updateSummary?.version }) }}
        </AppButton>
      </div>
      <p
        v-if="statusMessage"
        class="mt-3 text-xs"
        :class="phase === 'error' ? 'text-danger' : 'text-fg-muted'"
        role="status"
      >
        {{ statusMessage }}
      </p>
    </SettingsRow>
  </SettingsGroup>
</template>
