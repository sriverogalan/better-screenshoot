<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { SYSTEM_REPLACEMENT_HOTKEYS } from "@better-screenshoot/shared-types";
import type { SystemScreenshotShortcut } from "../../lib/tauri";
import { formatHotkey } from "../../lib/format-hotkey";
import { systemShortcutLabelKey } from "../../lib/system-shortcut-labels";
import AppDialog from "../ui/AppDialog.vue";
import AppButton from "../ui/AppButton.vue";
import AppBadge from "../ui/AppBadge.vue";
import AppKbd from "../ui/AppKbd.vue";

defineProps<{
  open: boolean;
  busy: boolean;
  shortcuts: SystemScreenshotShortcut[];
}>();

const emit = defineEmits<{
  close: [];
  confirm: [];
}>();

const { t } = useI18n();

const replacements = computed(() => [
  {
    hotkey: formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_screen),
    action: t("history.captureScreen"),
  },
  {
    hotkey: formatHotkey(SYSTEM_REPLACEMENT_HOTKEYS.capture_area),
    action: t("history.captureRegion"),
  },
]);

function shortcutLabel(shortcut: SystemScreenshotShortcut) {
  const key = systemShortcutLabelKey(shortcut.id);
  return key ? t(key) : shortcut.label;
}
</script>

<template>
  <AppDialog
    :open="open"
    :title="t('systemScreenshotDialog.title')"
    :description="t('systemScreenshotDialog.description')"
    :busy="busy"
    @close="emit('close')"
  >
    <ul class="space-y-2 rounded-lg border border-sep bg-win px-3 py-3 text-sm">
      <li
        v-for="item in replacements"
        :key="item.hotkey"
        class="flex items-center justify-between gap-3"
      >
        <span class="text-fg-muted">{{ item.action }}</span>
        <AppKbd>{{ item.hotkey }}</AppKbd>
      </li>
    </ul>

    <ul
      v-if="shortcuts.length > 0"
      class="mt-3 space-y-2 rounded-lg border border-sep bg-win px-3 py-3 text-sm"
    >
      <li
        v-for="shortcut in shortcuts"
        :key="shortcut.id"
        class="flex items-center justify-between gap-3"
      >
        <span class="text-fg-muted">{{ shortcutLabel(shortcut) }}</span>
        <AppBadge :tone="shortcut.enabled ? 'warning' : 'success'">
          {{
            shortcut.enabled
              ? t("settings.activeOnMacos")
              : t("systemScreenshotDialog.willBeDisabled")
          }}
        </AppBadge>
      </li>
    </ul>

    <template #footer>
      <AppButton variant="secondary" :disabled="busy" @click="emit('close')">
        {{ t("common.cancel") }}
      </AppButton>
      <AppButton variant="primary" :disabled="busy" @click="emit('confirm')">
        {{
          busy
            ? t("common.replacing")
            : t("systemScreenshotDialog.confirm")
        }}
      </AppButton>
    </template>
  </AppDialog>
</template>
