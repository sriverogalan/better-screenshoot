<script setup lang="ts">
import { IconPhoto, IconSettings } from "@tabler/icons-vue";

const navItems = [
  { to: "/history", label: "Historial", icon: IconPhoto },
  { to: "/settings", label: "Ajustes", icon: IconSettings },
] as const;
</script>

<template>
  <nav
    class="flex w-52 shrink-0 flex-col border-r border-border bg-surface-raised"
    aria-label="Navegación principal"
  >
    <div class="flex items-center gap-2 border-b border-border px-4 py-4">
      <IconPhoto class="size-5 text-accent" aria-hidden="true" />
      <span class="text-sm font-semibold">Better Screenshoot</span>
    </div>

    <ul class="flex flex-1 flex-col gap-1 p-3">
      <li v-for="item in navItems" :key="item.to">
        <RouterLink
          :to="item.to"
          class="flex items-center gap-2 rounded-lg px-3 py-2 text-sm text-text-muted transition hover:bg-surface hover:text-text"
          active-class="!bg-surface !text-text"
        >
          <component :is="item.icon" class="size-4 shrink-0" aria-hidden="true" />
          {{ item.label }}
        </RouterLink>
      </li>
    </ul>
  </nav>
</template>
