<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { IconCircleCheckFilled, IconLoader2 } from "@tabler/icons-vue";
import appIcon from "../../assets/app-icon.png";
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

const downloadPercent = computed(() => {
  if (phase.value !== "downloading" || !totalBytes.value) return null;
  return Math.min(100, Math.round((downloadedBytes.value / totalBytes.value) * 100));
});

const statusMessage = computed(() => {
  if (phase.value === "checking") {
    return t("settings.updates.status.checking");
  }

  if (phase.value === "downloading") {
    if (downloadPercent.value !== null) {
      return t("settings.updates.status.downloadingWithPercent", {
        percent: downloadPercent.value,
      });
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
  <SettingsGroup
    :label="t('settings.updates.title')"
    :footer="t('settings.updates.description')"
  >
    <SettingsRow>
      <div class="flex min-w-0 flex-1 items-center gap-3 py-1">
        <img :src="appIcon" alt="" class="size-12 shrink-0" draggable="false" />
        <div class="min-w-0">
          <p class="text-body font-semibold text-fg">{{ t("common.appName") }}</p>
          <p class="text-subhead tabular-nums text-fg-muted">
            {{ t("settings.updates.currentVersion") }}
            {{ currentVersion ?? "…" }}
          </p>
          <p
            v-if="statusMessage"
            class="mt-1 flex items-center gap-1 text-subhead"
            :class="phase === 'error' ? 'text-danger' : 'text-fg-muted'"
            role="status"
          >
            <IconLoader2 v-if="isBusy" class="size-3.5 animate-spin" aria-hidden="true" />
            <IconCircleCheckFilled
              v-else-if="statusCode === 'latest'"
              class="size-3.5 text-success"
              aria-hidden="true"
            />
            {{ statusMessage }}
          </p>
        </div>
      </div>
      <div class="flex shrink-0 flex-col items-end gap-2">
        <AppButton
          v-if="updateAvailable"
          variant="primary"
          :disabled="isBusy"
          @click="installAvailableUpdate"
        >
          {{ t("settings.updates.installVersion", { version: updateSummary?.version }) }}
        </AppButton>
        <AppButton v-else :disabled="isBusy" @click="checkForUpdates()">
          {{ t("settings.updates.checkForUpdates") }}
        </AppButton>
      </div>
    </SettingsRow>
    <div v-if="downloadPercent !== null" class="px-3 pb-3">
      <div class="h-1 overflow-hidden rounded-full bg-hover">
        <div
          class="h-full rounded-full bg-accent transition-[width] duration-300"
          :style="{ width: `${downloadPercent}%` }"
        />
      </div>
    </div>
  </SettingsGroup>
</template>
