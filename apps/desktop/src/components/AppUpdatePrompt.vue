<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { IconX } from "@tabler/icons-vue";
import appIcon from "../assets/app-icon.png";
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
    class="fixed right-3 top-3 z-50 w-[340px] rounded-2xl bg-material p-3 text-body text-fg shadow-popover backdrop-blur-2xl backdrop-saturate-150"
    role="status"
    aria-live="polite"
  >
    <div class="flex items-start gap-3">
      <img :src="appIcon" alt="" class="size-9 shrink-0" draggable="false" />
      <div class="min-w-0 flex-1">
        <p class="font-semibold">
          {{ t("settings.updates.status.available", { version: updateSummary?.version }) }}
        </p>
        <p v-if="updateSummary?.notes" class="mt-0.5 line-clamp-3 text-subhead text-fg-muted">
          {{ updateSummary.notes }}
        </p>
        <p v-if="progressLabel" class="mt-0.5 text-subhead text-accent">
          {{ progressLabel }}
        </p>
        <div class="mt-2.5 flex flex-wrap gap-2">
          <AppButton variant="primary" size="sm" :disabled="isDownloading" @click="installAvailableUpdate">
            {{
              isDownloading
                ? t("settings.updates.downloadingAction")
                : t("settings.updates.updateNow")
            }}
          </AppButton>
          <AppButton size="sm" :disabled="isDownloading" @click="dismissPrompt">
            {{ t("settings.updates.later") }}
          </AppButton>
        </div>
      </div>
      <button
        type="button"
        class="flex size-5 items-center justify-center rounded-full text-fg-muted hover:bg-hover hover:text-fg disabled:opacity-40"
        :disabled="isDownloading"
        :aria-label="t('settings.updates.dismissNotification')"
        @click="dismissPrompt"
      >
        <IconX class="size-3.5" stroke-width="2.25" aria-hidden="true" />
      </button>
    </div>
  </div>
</template>
