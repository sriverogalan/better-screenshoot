<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { IconX } from "@tabler/icons-vue";
import { SYSTEM_REPLACEMENT_HOTKEYS } from "@better-screenshoot/shared-types";
import { nextTick, onUnmounted, ref, watch } from "vue";
import type { SystemScreenshotShortcut } from "../../lib/tauri";
import { formatHotkey } from "../../lib/format-hotkey";
import { systemShortcutLabelKey } from "../../lib/system-shortcut-labels";

const props = defineProps<{
  open: boolean;
  busy: boolean;
  shortcuts: SystemScreenshotShortcut[];
}>();

const emit = defineEmits<{
  close: [];
  confirm: [];
}>();

const { t } = useI18n();
const closeButtonRef = ref<HTMLButtonElement | null>(null);

const replacements = computed(() => [
  {
    hotkey: formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_screen),
    action: t("history.captureScreen"),
  },
  {
    hotkey: formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_area),
    action: t("history.captureRegion"),
  },
  {
    hotkey: formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_window),
    action: t("history.captureWindow"),
  },
]);

function shortcutLabel(shortcut: SystemScreenshotShortcut) {
  const key = systemShortcutLabelKey(shortcut.id);
  return key ? t(key) : shortcut.label;
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && !props.busy) {
    emit("close");
  }
}

watch(
  () => props.open,
  async (isOpen) => {
    if (!isOpen) return;
    await nextTick();
    closeButtonRef.value?.focus();
  },
);

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      document.addEventListener("keydown", onKeydown);
      return;
    }
    document.removeEventListener("keydown", onKeydown);
  },
);

onUnmounted(() => {
  document.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      @click.self="!busy && emit('close')"
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="system-screenshot-permission-title"
        aria-describedby="system-screenshot-permission-description"
        class="w-full max-w-lg rounded-xl border border-border bg-[#1a1d24] p-5 shadow-2xl"
      >
        <div class="mb-4 flex items-start justify-between gap-4">
          <div>
            <h2
              id="system-screenshot-permission-title"
              class="text-base font-semibold text-text"
            >
              {{ t("systemScreenshotDialog.title") }}
            </h2>
            <p
              id="system-screenshot-permission-description"
              class="mt-1 text-sm text-text-muted"
            >
              {{ t("systemScreenshotDialog.description") }}
            </p>
          </div>
          <button
            ref="closeButtonRef"
            type="button"
            class="rounded-lg p-1 text-text-muted hover:bg-border hover:text-text disabled:opacity-50"
            :aria-label="t('common.close')"
            :disabled="busy"
            @click="emit('close')"
          >
            <IconX class="size-5" />
          </button>
        </div>

        <ul class="space-y-2 rounded-lg border border-border bg-surface px-3 py-3 text-sm">
          <li
            v-for="item in replacements"
            :key="item.hotkey"
            class="flex items-center justify-between gap-3"
          >
            <span class="text-text-muted">{{ item.action }}</span>
            <kbd class="rounded-md border border-border bg-surface-raised px-2 py-0.5 font-mono text-xs">
              {{ item.hotkey }}
            </kbd>
          </li>
        </ul>

        <ul
          v-if="shortcuts.length > 0"
          class="mt-3 space-y-2 rounded-lg border border-border bg-surface px-3 py-3 text-sm"
        >
          <li
            v-for="shortcut in shortcuts"
            :key="shortcut.id"
            class="flex items-center justify-between gap-3"
          >
            <span class="text-text-muted">{{ shortcutLabel(shortcut) }}</span>
            <span
              class="rounded-md px-2 py-0.5 text-xs"
              :class="
                shortcut.enabled
                  ? 'bg-amber-950/50 text-amber-100'
                  : 'bg-emerald-950/50 text-emerald-100'
              "
            >
              {{
                shortcut.enabled
                  ? t("settings.activeOnMacos")
                  : t("systemScreenshotDialog.willBeDisabled")
              }}
            </span>
          </li>
        </ul>

        <div class="mt-5 flex flex-wrap justify-end gap-2">
          <button
            type="button"
            class="rounded-lg border border-border px-4 py-2 text-sm hover:bg-border disabled:opacity-50"
            :disabled="busy"
            @click="emit('close')"
          >
            {{ t("common.cancel") }}
          </button>
          <button
            type="button"
            class="rounded-lg bg-accent px-4 py-2 text-sm text-white hover:bg-accent-hover disabled:opacity-50"
            :disabled="busy"
            @click="emit('confirm')"
          >
            {{
              busy
                ? t("common.replacing")
                : t("systemScreenshotDialog.confirm")
            }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
