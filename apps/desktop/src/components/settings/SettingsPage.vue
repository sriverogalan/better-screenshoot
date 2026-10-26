<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useRoute } from "vue-router";
import { findNavItem } from "../../lib/navigation";
import PageScaffold from "../ui/PageScaffold.vue";
import TintedIcon from "../ui/TintedIcon.vue";
import PendingCaptureBanner from "../PendingCaptureBanner.vue";

const { t } = useI18n();
const route = useRoute();

const navItem = computed(() => findNavItem(route.path));
const title = computed(() => (navItem.value ? t(navItem.value.labelKey) : t("settings.title")));
</script>

<template>
  <PageScaffold :title="title">
    <div class="mx-auto w-full max-w-[620px] px-6 pb-10 pt-1">
      <PendingCaptureBanner />

      <section
        v-if="navItem"
        class="mb-6 flex flex-col items-center rounded-[var(--radius-group)] bg-group px-6 py-5 text-center shadow-[inset_0_0_0_.5px_var(--color-sep)]"
      >
        <TintedIcon :icon="navItem.icon" :tint="navItem.tint" size="xl" />
        <h2 class="mt-3 text-title2 font-bold text-fg">{{ title }}</h2>
        <p
          v-if="navItem.descriptionKey"
          class="mt-1 max-w-[420px] text-body text-fg-muted"
        >
          {{ t(navItem.descriptionKey) }}
        </p>
      </section>

      <slot />
    </div>
  </PageScaffold>
</template>
