<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { formatHotkey } from "../../lib/format-hotkey";
import { parseHotkeyEvent } from "../../lib/parse-hotkey-event";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    disabled?: boolean;
    lockedLabel?: string;
    ariaLabel?: string;
  }>(),
  {
    disabled: false,
    lockedLabel: undefined,
    ariaLabel: undefined,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const { t } = useI18n();
const recording = ref(false);

const displayValue = computed(() => {
  if (props.lockedLabel) return props.lockedLabel;
  if (!props.modelValue) return t("settings.hotkeys.pressShortcut");
  return formatHotkey(props.modelValue);
});

function startRecording() {
  if (props.disabled) return;
  recording.value = true;
}

function stopRecording() {
  recording.value = false;
}

function onKeydown(event: KeyboardEvent) {
  if (!recording.value || props.disabled) return;

  event.preventDefault();
  event.stopPropagation();

  if (event.key === "Escape") {
    stopRecording();
    return;
  }

  const accelerator = parseHotkeyEvent(event);
  if (!accelerator) return;

  emit("update:modelValue", accelerator);
  stopRecording();
}

function onBlur() {
  stopRecording();
}
</script>

<template>
  <button
    type="button"
    class="inline-flex h-[24px] min-w-[132px] shrink-0 items-center justify-center rounded-[var(--radius-control)] px-3 text-body outline-none transition-[background-color,box-shadow] duration-150 disabled:opacity-50"
    :class="
      recording
        ? 'bg-accent/15 text-accent shadow-[0_0_0_2px_var(--color-accent)]'
        : 'bg-field shadow-control enabled:hover:bg-control-hover'
    "
    :disabled="disabled"
    :aria-pressed="recording"
    :aria-label="ariaLabel"
    :title="disabled ? undefined : t('settings.hotkeys.clickToRecord')"
    @click="startRecording"
    @keydown="onKeydown"
    @blur="onBlur"
  >
    <span
      v-if="recording"
      class="flex items-center gap-1.5 text-subhead font-medium"
    >
      <span class="size-1.5 animate-pulse rounded-full bg-accent" aria-hidden="true" />
      {{ t("settings.hotkeys.pressKeys") }}
    </span>
    <span
      v-else
      class="font-medium tracking-[0.12em]"
      :class="modelValue || lockedLabel ? 'text-fg' : 'tracking-normal text-fg-muted'"
    >
      {{ displayValue }}
    </span>
  </button>
</template>
