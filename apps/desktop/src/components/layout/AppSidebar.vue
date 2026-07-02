<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { IconPhoto, IconSettings } from "@tabler/icons-vue";
import logoUrl from "../../assets/logo.svg";

const { t } = useI18n();

const navItems = computed(() => [
  { to: "/history", label: t("nav.history"), icon: IconPhoto },
  { to: "/settings", label: t("nav.settings"), icon: IconSettings },
]);
</script>

<template>
  <nav
    class="flex min-h-full w-52 shrink-0 self-stretch flex-col border-r border-border bg-surface-raised"
    :aria-label="t('nav.mainNavigation')"
  >
    <div class="flex items-center gap-2 border-b border-border px-4 py-4">
      <img :src="logoUrl" alt="" class="size-5 shrink-0" aria-hidden="true" />
      <span class="text-sm font-semibold">{{ t("common.appName") }}</span>
    </div>

    <ul class="flex min-h-0 flex-1 flex-col gap-1 overflow-hidden p-3">
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
