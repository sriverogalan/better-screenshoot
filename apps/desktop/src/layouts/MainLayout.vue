<script setup lang="ts">
import AppSidebar from "../components/layout/AppSidebar.vue";
</script>

<template>
  <div class="flex h-full min-h-0">
    <AppSidebar />
    <main class="min-h-0 flex-1 overflow-y-auto">
      <RouterView />
    </main>
  </div>
</template>
