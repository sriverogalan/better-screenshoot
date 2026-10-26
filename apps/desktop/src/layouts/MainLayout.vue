<script setup lang="ts">
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import appIcon from "../assets/app-icon.png";
import { LIBRARY_NAV, SETTINGS_NAV, type NavItem } from "../lib/navigation";
import { useAppUpdater } from "../composables/useAppUpdater";
import TintedIcon from "../components/ui/TintedIcon.vue";

const { t } = useI18n();
const route = useRoute();
const { currentVersion, loadCurrentVersion } = useAppUpdater();

const sections: Array<{ labelKey?: string; items: NavItem[] }> = [
  { items: LIBRARY_NAV },
  { labelKey: "nav.settings", items: SETTINGS_NAV },
];

function isActive(item: NavItem) {
  return route.path === item.path;
}

onMounted(() => {
  void loadCurrentVersion();
});
</script>

<template>
  <div class="flex h-full">
    <aside
      class="flex w-[220px] shrink-0 flex-col bg-sidebar shadow-[inset_-.5px_0_0_var(--color-sep)]"
    >
      <div class="flex items-center gap-2.5 px-4 pb-3 pt-3">
        <img :src="appIcon" alt="" class="size-8 shrink-0" draggable="false" />
        <div class="min-w-0">
          <p class="truncate text-body font-semibold text-fg">{{ t("common.appName") }}</p>
          <p class="text-caption tabular-nums text-fg-muted">
            {{ currentVersion ? `v${currentVersion}` : "" }}
          </p>
        </div>
      </div>

      <nav
        class="min-h-0 flex-1 overflow-y-auto px-2.5 pb-4"
        :aria-label="t('nav.mainNavigation')"
      >
        <template v-for="(section, index) in sections" :key="index">
          <p
            v-if="section.labelKey"
            class="px-2 pb-1 pt-4 text-caption font-semibold text-fg-muted"
          >
            {{ t(section.labelKey) }}
          </p>
          <ul class="space-y-px">
            <li v-for="item in section.items" :key="item.path">
              <RouterLink
                :to="item.path"
                class="flex h-7 items-center gap-2 rounded-[6px] px-1.5 text-body outline-none transition-colors duration-100"
                :class="
                  isActive(item)
                    ? 'bg-accent text-white'
                    : 'text-fg hover:bg-hover'
                "
                :aria-current="isActive(item) ? 'page' : undefined"
                draggable="false"
              >
                <TintedIcon :icon="item.icon" :tint="item.tint" />
                <span class="truncate">{{ t(item.labelKey) }}</span>
              </RouterLink>
            </li>
          </ul>
        </template>
      </nav>
    </aside>

    <main class="relative min-w-0 flex-1 bg-content">
      <RouterView />
    </main>
  </div>
</template>
