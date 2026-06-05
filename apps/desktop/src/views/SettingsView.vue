<script setup lang="ts">
import { computed, ref } from "vue";
import { TIERS } from "@better-screenshoot/licensing";
import { useSettingsStore } from "../stores/settings";
import type { HotkeyConfig, LicenseTier } from "@better-screenshoot/shared-types";
import { validateLicenseKey } from "../lib/tauri";

const settingsStore = useSettingsStore();

const hotkeyFields: Array<{ key: keyof HotkeyConfig; label: string; hint?: string }> = [
  { key: "capture_area", label: "Capturar región", hint: "En macOS abre el selector nativo del sistema" },
  { key: "capture_screen", label: "Capturar pantalla" },
  { key: "capture_window", label: "Capturar ventana" },
  { key: "open_history", label: "Abrir historial" },
];

const settings = computed(() => settingsStore.settings);
const licenseKey = ref("");
const licenseMessage = ref("");

async function applyLicense() {
  const result = await validateLicenseKey(licenseKey.value);
  licenseMessage.value = result.message;
  if (result.valid) {
    await settingsStore.save({
      ...settings.value,
      tier: result.tier as LicenseTier,
    });
  }
}

const tierLabels: Record<LicenseTier, string> = {
  community: "Community (gratis)",
  pro: "Pro",
  cloud: "Cloud",
  team: "Team",
};

async function updateField<K extends keyof typeof settings.value>(
  key: K,
  value: (typeof settings.value)[K],
) {
  await settingsStore.save({ ...settings.value, [key]: value });
}

async function updateHotkey(
  key: keyof typeof settings.value.hotkeys,
  value: string,
) {
  await settingsStore.save({
    ...settings.value,
    hotkeys: { ...settings.value.hotkeys, [key]: value },
  });
}
</script>

<template>
  <div class="flex min-h-full flex-col">
    <header class="border-b border-border px-6 py-4">
      <h1 class="text-lg font-semibold">Ajustes</h1>
    </header>

    <main class="mx-auto w-full max-w-2xl flex-1 space-y-8 p-6">
      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">Captura</h2>
        <div class="space-y-4 rounded-xl border border-border bg-surface-raised p-4">
          <label class="block">
            <span class="mb-1 block text-sm">Carpeta de guardado</span>
            <input
              :value="settings.save_directory"
              type="text"
              class="w-full rounded-lg border border-border bg-surface px-3 py-2 text-sm"
              @change="updateField('save_directory', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <label class="flex items-center gap-3">
            <input
              :checked="settings.auto_copy"
              type="checkbox"
              class="size-4 rounded border-border"
              @change="updateField('auto_copy', ($event.target as HTMLInputElement).checked)"
            />
            <span class="text-sm">Copiar al portapapeles automáticamente</span>
          </label>
          <label class="flex items-center gap-3">
            <input
              :checked="settings.auto_save"
              type="checkbox"
              class="size-4 rounded border-border"
              @change="updateField('auto_save', ($event.target as HTMLInputElement).checked)"
            />
            <span class="text-sm">Guardar capturas en disco</span>
          </label>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">Integraciones</h2>
        <div class="space-y-4 rounded-xl border border-border bg-surface-raised p-4">
          <label class="flex items-center gap-3">
            <input
              :checked="settings.allow_external_control"
              type="checkbox"
              class="size-4 rounded border-border"
              @change="
                updateField(
                  'allow_external_control',
                  ($event.target as HTMLInputElement).checked,
                )
              "
            />
            <span class="text-sm">Permitir control externo (Raycast, CLI, URL scheme)</span>
          </label>
          <p class="text-xs text-text-muted">
            URL scheme: <code class="text-accent">betterscreenshoot://capture-area</code>
          </p>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">Sustituir capturas del sistema</h2>
        <div class="space-y-4 rounded-xl border border-border bg-surface-raised p-4">
          <label class="flex items-start gap-3">
            <input
              :checked="settings.replace_system_screenshots"
              type="checkbox"
              class="mt-0.5 size-4 rounded border-border"
              @change="
                updateField(
                  'replace_system_screenshots',
                  ($event.target as HTMLInputElement).checked,
                )
              "
            />
            <span class="text-sm">
              Usar Better Screenshoot como herramienta principal de captura
            </span>
          </label>
          <p class="text-xs leading-relaxed text-text-muted">
            macOS no permite desactivar los atajos del sistema desde una app. Si activas esta
            opción, asigna aquí atajos que no choquen con
            <code class="text-accent">⌘⇧3</code>,
            <code class="text-accent">⌘⇧4</code> y
            <code class="text-accent">⌘⇧5</code>, y desactívalos en
            Ajustes del Sistema → Teclado → Capturas de pantalla.
          </p>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">Atajos globales</h2>
        <div class="space-y-3 rounded-xl border border-border bg-surface-raised p-4">
          <label v-for="field in hotkeyFields" :key="field.key" class="block">
            <span class="mb-1 block text-sm">
              {{ field.label }}
              <span v-if="field.hint" class="text-text-muted"> — {{ field.hint }}</span>
            </span>
            <input
              :value="settings.hotkeys[field.key]"
              type="text"
              class="w-full rounded-lg border border-border bg-surface px-3 py-2 font-mono text-sm"
              @change="updateHotkey(field.key, ($event.target as HTMLInputElement).value)"
            />
          </label>
        </div>
      </section>

      <section>
        <h2 class="mb-4 text-sm font-medium text-text-muted">Licencia</h2>
        <div class="space-y-4 rounded-xl border border-border bg-surface-raised p-4">
          <p class="text-sm">
            Plan actual:
            <span class="font-medium text-accent">{{ tierLabels[settings.tier] }}</span>
          </p>
          <label class="block">
            <span class="mb-1 block text-sm">Clave de licencia</span>
            <input
              v-model="licenseKey"
              type="text"
              placeholder="BS-PRO-..."
              class="w-full rounded-lg border border-border bg-surface px-3 py-2 font-mono text-sm"
            />
          </label>
          <button
            type="button"
            class="rounded-lg bg-accent px-4 py-2 text-sm text-white hover:bg-accent-hover"
            @click="applyLicense"
          >
            Activar licencia
          </button>
          <p v-if="licenseMessage" class="text-xs text-text-muted">{{ licenseMessage }}</p>
          <ul class="space-y-2 text-xs text-text-muted">
            <li v-for="tier in Object.values(TIERS)" :key="tier.id">
              <strong class="text-text">{{ tier.name }}</strong> — {{ tier.price }}:
              {{ tier.features.slice(0, 2).join(", ") }}…
            </li>
          </ul>
        </div>
      </section>
    </main>
  </div>
</template>
