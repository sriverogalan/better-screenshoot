<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { IconDownload, IconX } from "@tabler/icons-vue";
import { useAppUpdater } from "../composables/useAppUpdater";
import AppButton from "./ui/AppButton.vue";

const { t } = useI18n();

const {
  updateSummary,
  phase,
  downloadedBytes,
  totalBytes,
  installAvailableUpdate,
  dismissPrompt,
} = useAppUpdater();

const isDownloading = computed(() => phase.value === "downloading");

const progressLabel = computed(() => {
  if (!isDownloading.value) {
    return null;
  }

  if (totalBytes.value) {
    const percent = Math.min(
      100,
      Math.round((downloadedBytes.value / totalBytes.value) * 100),
    );
    return t("settings.updates.status.downloadingWithPercent", { percent });
  }

  return t("settings.updates.status.downloading");
});
</script>

<template>
  <div
    class="fixed inset-x-4 top-4 z-50 rounded-xl border border-accent/40 bg-elev px-4 py-3 text-sm text-fg shadow-window"
    role="status"
    aria-live="polite"
  >
    <div class="flex items-start gap-3">
      <IconDownload class="mt-0.5 size-5 shrink-0 text-accent" aria-hidden="true" />
      <div class="min-w-0 flex-1 space-y-2">
        <p class="font-medium">
          {{ t("settings.updates.status.available", { version: updateSummary?.version }) }}
        </p>
        <p v-if="updateSummary?.notes" class="text-xs text-fg-muted">
          {{ updateSummary.notes }}
        </p>
        <p v-if="progressLabel" class="text-xs text-accent">
          {{ progressLabel }}
        </p>
        <div class="flex flex-wrap gap-2">
          <AppButton variant="primary" :disabled="isDownloading" @click="installAvailableUpdate">
            {{
              isDownloading
                ? t("settings.updates.downloadingAction")
                : t("settings.updates.updateNow")
            }}
          </AppButton>
          <AppButton variant="secondary" :disabled="isDownloading" @click="dismissPrompt">
            {{ t("settings.updates.later") }}
          </AppButton>
        </div>
      </div>
      <button
        type="button"
        class="rounded-md p-1 text-fg-muted hover:bg-sep/40 hover:text-fg disabled:opacity-50"
        :disabled="isDownloading"
        :aria-label="t('settings.updates.dismissNotification')"
        @click="dismissPrompt"
      >
        <IconX class="size-4" aria-hidden="true" />
      </button>
    </div>
  </div>
</template>
