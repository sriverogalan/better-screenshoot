<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import { IconFolderFilled } from "@tabler/icons-vue";
import { useSettingsStore } from "../../stores/settings";
import { pickDirectory } from "../../lib/tauri";
import SettingsGroup from "../ui/SettingsGroup.vue";
import SettingsRow from "../ui/SettingsRow.vue";
import AppToggle from "../ui/AppToggle.vue";
import AppButton from "../ui/AppButton.vue";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const picking = ref(false);

const settings = computed(() => settingsStore.settings);

const folderLabel = computed(() => {
  const path = settings.value.save_directory;
  if (!path) return t("settings.saveFolder");
  const parts = path.split(/[/\\]/).filter(Boolean);
  if (parts.length <= 2) return path;
  return `…/${parts.slice(-2).join("/")}`;
});

async function updateField<K extends keyof typeof settings.value>(
  key: K,
  value: (typeof settings.value)[K],
) {
  await settingsStore.save({ ...settings.value, [key]: value });
}

async function browseFolder() {
  if (picking.value) return;
  picking.value = true;
  try {
    const picked = await pickDirectory(settings.value.save_directory);
    if (picked) {
      await updateField("save_directory", picked);
    }
  } finally {
    picking.value = false;
  }
}
</script>

<template>
  <SettingsGroup :label="t('settings.sections.afterCapture')">
    <SettingsRow :label="t('settings.autoCopy')">
      <AppToggle
        :model-value="settings.auto_copy"
        :label="t('settings.autoCopy')"
        @update:model-value="updateField('auto_copy', $event)"
      />
    </SettingsRow>
    <SettingsRow :label="t('settings.autoSave')">
      <AppToggle
        :model-value="settings.auto_save"
        :label="t('settings.autoSave')"
        @update:model-value="updateField('auto_save', $event)"
      />
    </SettingsRow>
  </SettingsGroup>

  <SettingsGroup
    :label="t('settings.saveFolder')"
    :footer="t('settings.saveFolderHint')"
  >
    <SettingsRow>
      <div class="flex min-w-0 flex-1 items-center gap-2.5">
        <IconFolderFilled class="size-7 shrink-0 text-sys-blue" aria-hidden="true" />
        <div class="min-w-0">
          <p class="truncate text-body text-fg" :title="settings.save_directory">
            {{ folderLabel }}
          </p>
          <p class="truncate text-caption text-fg-muted" :title="settings.save_directory">
            {{ settings.save_directory }}
          </p>
        </div>
      </div>
      <AppButton :disabled="picking" @click="browseFolder">
        {{ t("settings.chooseFolder") }}
      </AppButton>
    </SettingsRow>
  </SettingsGroup>
</template>
