<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import type { AppLocale } from "@better-screenshoot/shared-types";
import { useSettingsStore } from "../stores/settings";
import {
  getCaptureStatus,
  openScreenRecordingSettings,
  requestScreenCapturePermission,
  setLaunchAtLogin,
} from "../lib/tauri";
import { SUPPORTED_LOCALES, setLocale } from "../i18n";
import AppButton from "../components/ui/AppButton.vue";
import AppToggle from "../components/ui/AppToggle.vue";
import AlertBanner from "../components/ui/AlertBanner.vue";
import SettingsGroup from "../components/ui/SettingsGroup.vue";
import SettingsRow from "../components/ui/SettingsRow.vue";

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

async function handleOpenScreenRecordingSettings() {
  try {
    await requestScreenCapturePermission();
  } catch {
    // If macOS refuses to show the prompt, the settings pane is still opened below.
  }

  try {
    await openScreenRecordingSettings();
  } catch {
    // Polling still lets the user continue after granting the permission manually.
  } finally {
    await checkCaptureStatus();
    startPermissionPolling();
  }
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
  <div class="flex min-h-full flex-col bg-win text-fg">
    <!-- Wizard header -->
    <header class="flex items-center justify-between border-b border-sep px-6 py-4">
      <span class="text-sm text-fg-muted">
        {{ t("onboarding.step", { current: currentStep, total: TOTAL_STEPS }) }}
      </span>
      <select
        :value="settings.locale"
        class="rounded-lg border border-sep bg-field px-2 py-1 text-sm text-fg"
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
        <p class="mb-8 text-fg-muted">{{ t("onboarding.stepWelcome.description") }}</p>
        <AppButton variant="primary" @click="nextStep">
          {{ t("onboarding.stepWelcome.next") }}
        </AppButton>
      </section>

      <!-- Step 2: Screen Recording -->
      <section v-else-if="currentStep === 2">
        <h1 class="mb-4 text-2xl font-bold">{{ t("onboarding.stepPermission.title") }}</h1>
        <p class="mb-6 text-fg-muted">{{ t("onboarding.stepPermission.description") }}</p>

        <AlertBanner
          v-if="captureMessageCode && captureMessageCode !== 'displaysDetected'"
          tone="warning"
          class="mb-6"
        >
          {{ t(`errors.${captureMessageCode}`, { count: 0 }) }}
        </AlertBanner>

        <div class="space-y-3">
          <AppButton variant="secondary" @click="handleOpenScreenRecordingSettings">
            {{ t("onboarding.stepPermission.openSettings") }}
          </AppButton>

          <div class="flex items-center gap-4">
            <AppButton
              variant="primary"
              :disabled="!permissionGranted"
              @click="nextStep"
            >
              {{ permissionGranted ? t("onboarding.stepPermission.next") : t("onboarding.stepPermission.waiting") }}
            </AppButton>

            <button
              v-if="showSkipLink"
              type="button"
              class="text-sm text-fg-muted underline hover:text-fg"
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
        <p class="mb-6 text-fg-muted">{{ t("onboarding.stepAutostart.description") }}</p>

        <SettingsGroup class="mb-8">
          <SettingsRow>
            <span class="text-sm">{{ t("onboarding.stepAutostart.toggle") }}</span>
            <AppToggle
              :model-value="autostartEnabled"
              @update:model-value="toggleAutostart"
            />
          </SettingsRow>
        </SettingsGroup>

        <AppButton variant="primary" @click="nextStep">
          {{ t("onboarding.stepAutostart.next") }}
        </AppButton>
      </section>

      <!-- Step 4: Quick Settings -->
      <section v-else-if="currentStep === 4">
        <h1 class="mb-4 text-2xl font-bold">{{ t("onboarding.stepQuickSettings.title") }}</h1>
        <p class="mb-6 text-fg-muted">{{ t("onboarding.stepQuickSettings.description") }}</p>

        <SettingsGroup class="mb-8">
          <SettingsRow>
            <span class="text-sm">{{ t("onboarding.stepQuickSettings.autoCopy") }}</span>
            <AppToggle
              :model-value="settings.auto_copy"
              @update:model-value="updateAutoCopy"
            />
          </SettingsRow>
          <SettingsRow>
            <span class="text-sm">{{ t("onboarding.stepQuickSettings.autoSave") }}</span>
            <AppToggle
              :model-value="settings.auto_save"
              @update:model-value="updateAutoSave"
            />
          </SettingsRow>
        </SettingsGroup>

        <AppButton variant="primary" @click="nextStep">
          {{ t("onboarding.stepQuickSettings.next") }}
        </AppButton>
      </section>

      <!-- Step 5: Done -->
      <section v-else-if="currentStep === 5">
        <h1 class="mb-4 text-2xl font-bold">{{ t("onboarding.stepDone.title") }}</h1>
        <p class="mb-8 text-fg-muted">{{ t("onboarding.stepDone.description") }}</p>
        <AppButton variant="primary" @click="completeOnboarding">
          {{ t("onboarding.stepDone.done") }}
        </AppButton>
      </section>

    </main>
  </div>
</template>
