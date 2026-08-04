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
  }>(),
  {
    disabled: false,
    lockedLabel: undefined,
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
    class="flex w-full items-center justify-between gap-3 rounded-[var(--radius-control)] border border-sep bg-field px-3 py-2 text-left text-sm outline-none transition-colors focus-visible:border-accent disabled:cursor-not-allowed disabled:opacity-60"
    :class="
      recording
        ? 'border-accent ring-2 ring-accent/30'
        : 'hover:border-fg-muted/40'
    "
    :disabled="disabled"
    :aria-pressed="recording"
    @click="startRecording"
    @keydown="onKeydown"
    @blur="onBlur"
  >
    <span
      class="font-mono"
      :class="modelValue || lockedLabel ? 'text-fg' : 'text-fg-muted'"
    >
      {{
        recording ? t("settings.hotkeys.recording") : displayValue
      }}
    </span>
    <span class="text-xs text-fg-muted">
      {{
        recording
          ? t("settings.hotkeys.pressKeys")
          : disabled
            ? ""
            : t("settings.hotkeys.clickToRecord")
      }}
    </span>
  </button>
</template>
