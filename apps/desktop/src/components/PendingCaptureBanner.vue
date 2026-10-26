<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { usePendingCaptureRecovery } from "../composables/usePendingCaptureRecovery";
import AlertBanner from "./ui/AlertBanner.vue";
import AppButton from "./ui/AppButton.vue";

const { t } = useI18n();
const {
  pendingCapture,
  recoveryBusy,
  recoveryError,
  openPendingInEditor,
} = usePendingCaptureRecovery();
</script>

<template>
  <AlertBanner v-if="pendingCapture" tone="info" class="mb-4">
    <p class="font-medium">
      {{ t("pendingCapture.title") }}
    </p>
    <p class="text-subhead text-fg-muted">
      {{
        t("pendingCapture.description", {
          width: pendingCapture.width,
          height: pendingCapture.height,
        })
      }}
    </p>
    <p v-if="recoveryError" class="text-subhead text-danger">
      {{ recoveryError }}
    </p>
    <template #actions>
      <AppButton
        variant="primary"
        size="sm"
        :disabled="recoveryBusy"
        @click="openPendingInEditor"
      >
        {{
          recoveryBusy ? t("common.opening") : t("pendingCapture.openInEditor")
        }}
      </AppButton>
    </template>
  </AlertBanner>
</template>
