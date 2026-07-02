<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import type { AppLocale } from "@better-screenshoot/shared-types";
import { useSettingsStore } from "../stores/settings";
import { getCaptureStatus, setLaunchAtLogin } from "../lib/tauri";
import { SUPPORTED_LOCALES, setLocale } from "../i18n";

const { t } = useI18n();
const router = useRouter();
const settingsStore = useSettingsStore();
const settings = computed(() => settingsStore.settings);

const TOTAL_STEPS = 5;
const currentStep = ref(1);

// Step 2: Screen recording permission polling
const captureMessageCode = ref<string | null>(null);
const permissionGranted = computed(() => captureMessageCode.value === "displaysDetected");
const showSkipLink = ref(false);

let pollInterval: ReturnType<typeof setInterval> | null = null;
let skipTimer: ReturnType<typeof setTimeout> | null = null;

function clearPermissionTimers() {
  if (pollInterval !== null) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
  if (skipTimer !== null) {
    clearTimeout(skipTimer);
    skipTimer = null;
  }
}

async function checkCaptureStatus() {
  try {
    const status = await getCaptureStatus();
    captureMessageCode.value = status.messageCode;
  } catch {
    // Silently ignore — polling will retry
  }
}

function startPermissionPolling() {
  clearPermissionTimers();
  showSkipLink.value = false;

  pollInterval = setInterval(async () => {
    await checkCaptureStatus();
  }, 2000);

  // Show skip link after 30s only if "macosPermissionGrantedNoDisplays"
  skipTimer = setTimeout(() => {
    if (captureMessageCode.value === "macosPermissionGrantedNoDisplays") {
      showSkipLink.value = true;
    }
  }, 30_000);
}

// Step 3: Launch at Login
const autostartEnabled = ref(true);

async function toggleAutostart(value: boolean) {
  autostartEnabled.value = value;
  try {
    await setLaunchAtLogin(value);
  } catch {
    // Non-critical — setting can be changed later
  }
}

// Step 4: Quick settings
async function updateAutoCopy(value: boolean) {
  await settingsStore.save({ ...settings.value, auto_copy: value });
}

async function updateAutoSave(value: boolean) {
  await settingsStore.save({ ...settings.value, auto_save: value });
}

// Language selector (all steps)
const languageOptions = computed(() =>
  SUPPORTED_LOCALES.map((locale) => ({
    value: locale,
    label: t(`settings.languageOptions.${locale}`),
  })),
);

async function onLocaleChange(locale: AppLocale) {
  await settingsStore.save({ ...settings.value, locale });
  await setLocale(locale);
}

// Step navigation
async function nextStep() {
  if (currentStep.value < TOTAL_STEPS) {
    currentStep.value += 1;
    if (currentStep.value === 2) {
      await checkCaptureStatus();
      startPermissionPolling();
    }
    if (currentStep.value === 3) {
      clearPermissionTimers();
      try {
        await setLaunchAtLogin(true);
        autostartEnabled.value = true;
      } catch {
        // Plugin may not be available in dev
      }
    }
  }
}

async function completeOnboarding() {
  await settingsStore.save({ ...settings.value, onboarding_completed: true });
  await router.replace("/settings");
}

async function skipPermissionStep() {
  clearPermissionTimers();
  currentStep.value += 1;
  if (currentStep.value === 3) {
    try {
      await setLaunchAtLogin(true);
      autostartEnabled.value = true;
    } catch {
      // Plugin may not be available in dev
    }
  }
}

// Watch captureMessageCode to stop polling once displays are detected
watch(captureMessageCode, (code) => {
  if (code === "displaysDetected") {
    clearPermissionTimers();
  }
  if (code === "macosPermissionGrantedNoDisplays" && skipTimer !== null) {
    // Skip timer is already counting — nothing extra needed here
  }
});

onMounted(async () => {
  if (currentStep.value === 2) {
    await checkCaptureStatus();
    startPermissionPolling();
  }
});

onUnmounted(() => {
  clearPermissionTimers();
});
</script>

<template>
  <div class="flex min-h-full flex-col bg-[#111318] text-[#e8eaed]">
    <!-- Wizard header -->
    <header class="flex items-center justify-between border-b border-border px-6 py-4">
      <span class="text-sm text-text-muted">
        {{ t("onboarding.step", { current: currentStep, total: TOTAL_STEPS }) }}
      </span>
      <select
        :value="settings.locale"
        class="rounded-lg border border-border bg-surface px-2 py-1 text-sm"
        :aria-label="t('onboarding.language')"
        @change="onLocaleChange(($event.target as HTMLSelectElement).value as AppLocale)"
      >
        <option
          v-for="option in languageOptions"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </option>
      </select>
    </header>

    <!-- Steps -->
    <main class="mx-auto w-full max-w-lg flex-1 space-y-8 p-8">

      <!-- Step 1: Welcome -->
      <section v-if="currentStep === 1">
        <h1 class="mb-4 text-2xl font-bold">{{ t("onboarding.stepWelcome.title") }}</h1>
        <p class="mb-8 text-text-muted">{{ t("onboarding.stepWelcome.description") }}</p>
        <button
          type="button"
          class="rounded-xl bg-accent px-6 py-3 font-medium text-white hover:bg-accent/90"
          @click="nextStep"
        >
          {{ t("onboarding.stepWelcome.next") }}
        </button>
      </section>

      <!-- Step 2: Screen Recording -->
      <section v-else-if="currentStep === 2">
        <h1 class="mb-4 text-2xl font-bold">{{ t("onboarding.stepPermission.title") }}</h1>
        <p class="mb-6 text-text-muted">{{ t("onboarding.stepPermission.description") }}</p>

        <div
          v-if="captureMessageCode && captureMessageCode !== 'displaysDetected'"
          class="mb-6 rounded-xl border border-amber-500/40 bg-amber-950/30 px-4 py-3 text-sm text-amber-100"
          role="status"
        >
          {{ t(`errors.${captureMessageCode}`, { count: 0 }) }}
        </div>

        <div class="space-y-3">
          <button
            type="button"
            class="rounded-xl border border-border bg-surface px-5 py-2.5 text-sm hover:bg-border/40"
            @click="startPermissionPolling"
          >
            {{ t("onboarding.stepPermission.openSettings") }}
          </button>

          <div class="flex items-center gap-4">
            <button
              type="button"
              class="rounded-xl bg-accent px-6 py-3 font-medium text-white hover:bg-accent/90 disabled:opacity-40 disabled:cursor-not-allowed"
              :disabled="!permissionGranted"
              @click="nextStep"
            >
              {{ permissionGranted ? t("onboarding.stepPermission.next") : t("onboarding.stepPermission.waiting") }}
            </button>

            <button
              v-if="showSkipLink"
              type="button"
              class="text-sm text-text-muted underline hover:text-[#e8eaed]"
              @click="skipPermissionStep"
            >
              {{ t("onboarding.stepPermission.skip") }}
            </button>
          </div>
        </div>
      </section>

      <!-- Step 3: Launch at Login -->
      <section v-else-if="currentStep === 3">
        <h1 class="mb-4 text-2xl font-bold">{{ t("onboarding.stepAutostart.title") }}</h1>
        <p class="mb-6 text-text-muted">{{ t("onboarding.stepAutostart.description") }}</p>

        <label class="mb-8 flex items-center gap-3">
          <input
            :checked="autostartEnabled"
            type="checkbox"
            class="size-4 rounded border-border"
            @change="toggleAutostart(($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm">{{ t("onboarding.stepAutostart.toggle") }}</span>
        </label>

        <button
          type="button"
          class="rounded-xl bg-accent px-6 py-3 font-medium text-white hover:bg-accent/90"
          @click="nextStep"
        >
          {{ t("onboarding.stepAutostart.next") }}
        </button>
      </section>

      <!-- Step 4: Quick Settings -->
      <section v-else-if="currentStep === 4">
        <h1 class="mb-4 text-2xl font-bold">{{ t("onboarding.stepQuickSettings.title") }}</h1>
        <p class="mb-6 text-text-muted">{{ t("onboarding.stepQuickSettings.description") }}</p>

        <div class="mb-8 space-y-4 rounded-xl border border-border bg-surface-raised p-4">
          <label class="flex items-center gap-3">
            <input
              :checked="settings.auto_copy"
              type="checkbox"
              class="size-4 rounded border-border"
              @change="updateAutoCopy(($event.target as HTMLInputElement).checked)"
            />
            <span class="text-sm">{{ t("onboarding.stepQuickSettings.autoCopy") }}</span>
          </label>
          <label class="flex items-center gap-3">
            <input
              :checked="settings.auto_save"
              type="checkbox"
              class="size-4 rounded border-border"
              @change="updateAutoSave(($event.target as HTMLInputElement).checked)"
            />
            <span class="text-sm">{{ t("onboarding.stepQuickSettings.autoSave") }}</span>
          </label>
        </div>

        <button
          type="button"
          class="rounded-xl bg-accent px-6 py-3 font-medium text-white hover:bg-accent/90"
          @click="nextStep"
        >
          {{ t("onboarding.stepQuickSettings.next") }}
        </button>
      </section>

      <!-- Step 5: Done -->
      <section v-else-if="currentStep === 5">
        <h1 class="mb-4 text-2xl font-bold">{{ t("onboarding.stepDone.title") }}</h1>
        <p class="mb-8 text-text-muted">{{ t("onboarding.stepDone.description") }}</p>
        <button
          type="button"
          class="rounded-xl bg-accent px-6 py-3 font-medium text-white hover:bg-accent/90"
          @click="completeOnboarding"
        >
          {{ t("onboarding.stepDone.done") }}
        </button>
      </section>

    </main>
  </div>
</template>
