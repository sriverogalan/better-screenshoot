<script setup lang="ts">
import { computed, onMounted } from "vue";
import { IconRefresh } from "@tabler/icons-vue";
import { useAppUpdater } from "../../composables/useAppUpdater";

const {
  phase,
  currentVersion,
  updateSummary,
  errorMessage,
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
    return "Checking for updates…";
  }

  if (phase.value === "downloading") {
    if (totalBytes.value) {
      const percent = Math.min(
        100,
        Math.round((downloadedBytes.value / totalBytes.value) * 100),
      );
      return `Downloading update… ${percent}%`;
    }

    return "Downloading update…";
  }

  if (updateAvailable.value && updateSummary.value) {
    return `Version ${updateSummary.value.version} is available.`;
  }

  return errorMessage.value;
});

onMounted(() => {
  void loadCurrentVersion();
});
</script>

<template>
  <section>
    <h2 class="mb-4 text-sm font-medium text-text-muted">Updates</h2>
    <div class="space-y-4 rounded-xl border border-border bg-surface-raised p-4">
      <p class="text-sm">
        Current version:
        <span class="font-medium text-accent">{{ currentVersion ?? "…" }}</span>
      </p>
      <p class="text-xs text-text-muted">
        The app checks for signed updates when it starts and from this screen.
      </p>

      <div class="flex flex-wrap gap-2">
        <button
          type="button"
          class="inline-flex items-center gap-2 rounded-lg border border-border bg-surface px-3 py-2 text-sm hover:bg-border disabled:opacity-50"
          :disabled="isBusy"
          @click="checkForUpdates()"
        >
          <IconRefresh class="size-4" aria-hidden="true" />
          Check for updates
        </button>
        <button
          v-if="updateAvailable"
          type="button"
          class="rounded-lg bg-accent px-3 py-2 text-sm text-white hover:bg-accent-hover disabled:opacity-50"
          :disabled="isBusy"
          @click="installAvailableUpdate"
        >
          Install v{{ updateSummary?.version }}
        </button>
      </div>

      <p
        v-if="statusMessage"
        class="text-xs"
        :class="phase === 'error' ? 'text-red-400' : 'text-text-muted'"
        role="status"
      >
        {{ statusMessage }}
      </p>
    </div>
  </section>
</template>
