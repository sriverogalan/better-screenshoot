<script setup lang="ts">
import { computed, onMounted, onUnmounted, type Component } from "vue";

export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: Component;
  danger?: boolean;
  separatorBefore?: boolean;
}

const props = defineProps<{
  x: number;
  y: number;
  items: ContextMenuItem[];
}>();

const emit = defineEmits<{
  select: [id: string];
  close: [];
}>();

const MENU_WIDTH = 200;
const ITEM_HEIGHT = 24;

const position = computed(() => {
  const height = props.items.length * ITEM_HEIGHT + 16;
  return {
    left: `${Math.max(8, Math.min(props.x, window.innerWidth - MENU_WIDTH - 8))}px`,
    top: `${Math.max(8, Math.min(props.y, window.innerHeight - height - 8))}px`,
  };
});

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.stopPropagation();
    emit("close");
  }
}

onMounted(() => window.addEventListener("keydown", onKeydown, true));
onUnmounted(() => window.removeEventListener("keydown", onKeydown, true));
</script>

<template>
  <div class="fixed inset-0 z-40" @click="emit('close')" @contextmenu.prevent="emit('close')" />
  <div
    class="context-menu fixed z-50 w-[200px] rounded-[9px] bg-material p-[5px] text-body shadow-popover backdrop-blur-2xl backdrop-saturate-150"
    :style="position"
    role="menu"
  >
    <template v-for="item in items" :key="item.id">
      <div v-if="item.separatorBefore" class="mx-2 my-[5px] h-px bg-sep" role="separator" />
      <button
        type="button"
        role="menuitem"
        class="group flex h-6 w-full items-center gap-2 rounded-[5px] px-2 text-left outline-none hover:bg-accent hover:text-white focus-visible:bg-accent focus-visible:text-white"
        :class="item.danger ? 'text-danger' : 'text-fg'"
        @click="emit('select', item.id)"
      >
        <component
          :is="item.icon"
          v-if="item.icon"
          class="size-4 shrink-0 opacity-80 group-hover:opacity-100"
          stroke-width="1.75"
        />
        <span class="truncate">{{ item.label }}</span>
      </button>
    </template>
  </div>
</template>

<style scoped>
.context-menu {
  animation: menu-in 120ms ease-out;
}

@keyframes menu-in {
  from {
    opacity: 0;
    transform: scale(0.97);
  }
}
</style>
