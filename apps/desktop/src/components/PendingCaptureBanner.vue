<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { usePendingCaptureRecovery } from "../composables/usePendingCaptureRecovery";

const { t } = useI18n();
const {
  pendingCapture,
  recoveryBusy,
  recoveryError,
  openPendingInEditor,
} = usePendingCaptureRecovery();
</script>

<template>
  <div
    v-if="pendingCapture"
    class="mb-4 rounded-xl border border-accent/40 bg-accent/10 px-4 py-3 text-sm"
    role="status"
  >
    <p class="font-medium text-text">
      {{ t("pendingCapture.title") }}
    </p>
    <p class="mt-1 text-xs text-text-muted">
      {{
        t("pendingCapture.description", {
          width: pendingCapture.width,
          height: pendingCapture.height,
        })
      }}
    </p>
    <p v-if="recoveryError" class="mt-2 text-xs text-red-400">
      {{ recoveryError }}
    </p>
    <button
      type="button"
      class="mt-3 rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:bg-accent-hover disabled:opacity-50"
      :disabled="recoveryBusy"
      @click="openPendingInEditor"
    >
      {{
        recoveryBusy ? t("common.opening") : t("pendingCapture.openInEditor")
      }}
    </button>
  </div>
</template>
