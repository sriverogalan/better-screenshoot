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
import {
  IconCircleCheckFilled,
  IconPower,
  IconScreenShare,
  IconSparkles,
} from "@tabler/icons-vue";
import appIcon from "../assets/app-icon.png";
import AppButton from "../components/ui/AppButton.vue";
import AppToggle from "../components/ui/AppToggle.vue";
import AppSelect from "../components/ui/AppSelect.vue";
import AlertBanner from "../components/ui/AlertBanner.vue";
import SettingsGroup from "../components/ui/SettingsGroup.vue";
import SettingsRow from "../components/ui/SettingsRow.vue";
import TintedIcon from "../components/ui/TintedIcon.vue";

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

async function onLocaleChange(value: string) {
  const locale = value as AppLocale;
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
  <div class="flex h-full flex-col bg-content text-fg">
    <header data-tauri-drag-region class="flex h-[52px] shrink-0 items-center justify-end px-4">
      <AppSelect
        :model-value="settings.locale"
        :options="languageOptions"
        :aria-label="t('onboarding.language')"
        @update:model-value="onLocaleChange"
      />
    </header>

    <main class="flex min-h-0 flex-1 flex-col items-center overflow-y-auto px-8">
      <Transition
        mode="out-in"
        enter-active-class="transition duration-300 ease-out"
        enter-from-class="translate-x-6 opacity-0"
        leave-active-class="transition duration-150 ease-in"
        leave-to-class="-translate-x-6 opacity-0"
      >
        <section
          :key="currentStep"
          class="my-auto flex w-full max-w-[460px] flex-col items-center py-6 text-center"
        >
          <img
            v-if="currentStep === 1"
            :src="appIcon"
            alt=""
            class="size-28 drop-shadow-[0_10px_24px_rgb(0_0_0_/_.3)]"
            draggable="false"
          />
          <TintedIcon
            v-else-if="currentStep === 2"
            :icon="IconScreenShare"
            tint="var(--color-sys-purple)"
            size="xl"
          />
          <TintedIcon
            v-else-if="currentStep === 3"
            :icon="IconPower"
            tint="var(--color-sys-green)"
            size="xl"
          />
          <TintedIcon
            v-else-if="currentStep === 4"
            :icon="IconSparkles"
            tint="var(--color-sys-orange)"
            size="xl"
          />
          <IconCircleCheckFilled
            v-else
            class="size-20 text-success"
            aria-hidden="true"
          />

          <!-- Step 1: Welcome -->
          <template v-if="currentStep === 1">
            <h1 class="mt-6 text-largetitle font-bold tracking-tight">
              {{ t("onboarding.stepWelcome.title") }}
            </h1>
            <p class="mt-2 text-title3 font-normal text-fg-muted">
              {{ t("onboarding.stepWelcome.description") }}
            </p>
          </template>

          <!-- Step 2: Screen Recording -->
          <template v-else-if="currentStep === 2">
            <h1 class="mt-6 text-title1 font-bold tracking-tight">
              {{ t("onboarding.stepPermission.title") }}
            </h1>
            <p class="mt-2 text-body text-fg-muted">
              {{ t("onboarding.stepPermission.description") }}
            </p>

            <AlertBanner
              v-if="captureMessageCode && captureMessageCode !== 'displaysDetected'"
              tone="warning"
              class="mt-6 w-full text-left"
            >
              {{ t(`errors.${captureMessageCode}`, { count: 0 }) }}
            </AlertBanner>

            <SettingsGroup class="mt-6 w-full text-left">
              <SettingsRow
                :label="t('onboarding.stepPermission.title')"
                :description="
                  permissionGranted
                    ? t('settings.capturePermissionImpact.granted')
                    : t('onboarding.stepPermission.waiting')
                "
              >
                <IconCircleCheckFilled
                  v-if="permissionGranted"
                  class="size-5 shrink-0 text-success"
                  aria-hidden="true"
                />
                <AppButton v-else @click="handleOpenScreenRecordingSettings">
                  {{ t("onboarding.stepPermission.openSettings") }}
                </AppButton>
              </SettingsRow>
            </SettingsGroup>

            <button
              v-if="showSkipLink"
              type="button"
              class="mt-4 text-subhead text-accent hover:underline"
              @click="skipPermissionStep"
            >
              {{ t("onboarding.stepPermission.skip") }}
            </button>
          </template>

          <!-- Step 3: Launch at Login -->
          <template v-else-if="currentStep === 3">
            <h1 class="mt-6 text-title1 font-bold tracking-tight">
              {{ t("onboarding.stepAutostart.title") }}
            </h1>
            <p class="mt-2 text-body text-fg-muted">
              {{ t("onboarding.stepAutostart.description") }}
            </p>

            <SettingsGroup class="mt-6 w-full text-left">
              <SettingsRow :label="t('onboarding.stepAutostart.toggle')">
                <AppToggle
                  :model-value="autostartEnabled"
                  :label="t('onboarding.stepAutostart.toggle')"
                  @update:model-value="toggleAutostart"
                />
              </SettingsRow>
            </SettingsGroup>
          </template>

          <!-- Step 4: Quick Settings -->
          <template v-else-if="currentStep === 4">
            <h1 class="mt-6 text-title1 font-bold tracking-tight">
              {{ t("onboarding.stepQuickSettings.title") }}
            </h1>
            <p class="mt-2 text-body text-fg-muted">
              {{ t("onboarding.stepQuickSettings.description") }}
            </p>

            <SettingsGroup class="mt-6 w-full text-left">
              <SettingsRow :label="t('onboarding.stepQuickSettings.autoCopy')">
                <AppToggle
                  :model-value="settings.auto_copy"
                  :label="t('onboarding.stepQuickSettings.autoCopy')"
                  @update:model-value="updateAutoCopy"
                />
              </SettingsRow>
              <SettingsRow :label="t('onboarding.stepQuickSettings.autoSave')">
                <AppToggle
                  :model-value="settings.auto_save"
                  :label="t('onboarding.stepQuickSettings.autoSave')"
                  @update:model-value="updateAutoSave"
                />
              </SettingsRow>
            </SettingsGroup>
          </template>

          <!-- Step 5: Done -->
          <template v-else-if="currentStep === 5">
            <h1 class="mt-6 text-largetitle font-bold tracking-tight">
              {{ t("onboarding.stepDone.title") }}
            </h1>
            <p class="mt-2 text-title3 font-normal text-fg-muted">
              {{ t("onboarding.stepDone.description") }}
            </p>
          </template>
        </section>
      </Transition>
    </main>

    <footer class="flex shrink-0 flex-col items-center gap-5 px-8 pb-8 pt-4">
      <AppButton
        v-if="currentStep === 1"
        variant="primary"
        size="lg"
        @click="nextStep"
      >
        {{ t("onboarding.stepWelcome.next") }}
      </AppButton>
      <AppButton
        v-else-if="currentStep === 2"
        variant="primary"
        size="lg"
        :disabled="!permissionGranted"
        @click="nextStep"
      >
        {{
          permissionGranted
            ? t("onboarding.stepPermission.next")
            : t("onboarding.stepPermission.waiting")
        }}
      </AppButton>
      <AppButton
        v-else-if="currentStep === 3"
        variant="primary"
        size="lg"
        @click="nextStep"
      >
        {{ t("onboarding.stepAutostart.next") }}
      </AppButton>
      <AppButton
        v-else-if="currentStep === 4"
        variant="primary"
        size="lg"
        @click="nextStep"
      >
        {{ t("onboarding.stepQuickSettings.next") }}
      </AppButton>
      <AppButton
        v-else
        variant="primary"
        size="lg"
        @click="completeOnboarding"
      >
        {{ t("onboarding.stepDone.done") }}
      </AppButton>

      <div
        class="flex items-center gap-2"
        role="progressbar"
        :aria-valuemin="1"
        :aria-valuemax="TOTAL_STEPS"
        :aria-valuenow="currentStep"
        :aria-label="t('onboarding.step', { current: currentStep, total: TOTAL_STEPS })"
      >
        <span
          v-for="step in TOTAL_STEPS"
          :key="step"
          class="h-[7px] rounded-full transition-all duration-300"
          :class="step === currentStep ? 'w-[18px] bg-fg' : 'w-[7px] bg-fg-tertiary'"
        />
      </div>
    </footer>
  </div>
</template>
