<script setup lang="ts">
import AppSidebar from "../components/layout/AppSidebar.vue";
</script>

<template>
  <div class="flex h-dvh max-h-dvh min-h-0 overflow-hidden overscroll-none bg-surface">
    <AppSidebar />
    <main class="min-h-0 flex-1 overflow-y-auto overscroll-contain bg-surface">
      <RouterView />
    </main>
  </div>
</template>
