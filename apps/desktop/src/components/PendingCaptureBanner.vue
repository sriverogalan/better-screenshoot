<script setup lang="ts">
import { usePendingCaptureRecovery } from "../composables/usePendingCaptureRecovery";

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
      Capture ready but editor did not open
    </p>
    <p class="mt-1 text-xs text-text-muted">
      {{ pendingCapture.width }}×{{ pendingCapture.height }} — temporarily saved until you edit or discard it.
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
      {{ recoveryBusy ? "Opening…" : "Open in editor" }}
    </button>
  </div>
</template>
