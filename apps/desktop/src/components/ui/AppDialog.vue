<script setup lang="ts">
import { nextTick, onUnmounted, ref, useId, watch } from "vue";

const props = withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    description?: string;
    busy?: boolean;
    closeOnOverlay?: boolean;
  }>(),
  {
    description: undefined,
    busy: false,
    closeOnOverlay: true,
  },
);

const emit = defineEmits<{
  close: [];
}>();

const panelRef = ref<HTMLElement | null>(null);
const titleId = useId();
const descriptionId = useId();

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape" && !props.busy) {
    emit("close");
  }
}

function onOverlayClick() {
  if (props.closeOnOverlay && !props.busy) {
    emit("close");
  }
}

watch(
  () => props.open,
  async (isOpen) => {
    if (!isOpen) {
      document.removeEventListener("keydown", onKeydown);
      return;
    }
    document.addEventListener("keydown", onKeydown);
    await nextTick();
    panelRef.value?.focus();
  },
);

onUnmounted(() => {
  document.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0"
      leave-active-class="transition duration-150 ease-in"
      leave-to-class="opacity-0"
    >
      <div
        v-if="open"
        class="fixed inset-0 z-50 flex items-start justify-center bg-black/25 px-4 pt-[12vh]"
        @click.self="onOverlayClick"
      >
        <div
          ref="panelRef"
          role="dialog"
          aria-modal="true"
          tabindex="-1"
          :aria-labelledby="titleId"
          :aria-describedby="description ? descriptionId : undefined"
          class="dialog-panel w-full max-w-[440px] rounded-xl bg-material p-5 text-fg shadow-popover outline-none backdrop-blur-2xl backdrop-saturate-150"
        >
          <h2 :id="titleId" class="text-title3 font-semibold">
            {{ title }}
          </h2>
          <p
            v-if="description"
            :id="descriptionId"
            class="mt-1.5 text-body text-fg-muted"
          >
            {{ description }}
          </p>

          <div v-if="$slots.default" class="mt-4">
            <slot />
          </div>

          <div
            v-if="$slots.footer"
            class="mt-5 flex flex-wrap justify-end gap-2"
          >
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.dialog-panel {
  animation: dialog-in 220ms cubic-bezier(0.2, 0.9, 0.3, 1.15);
}

@keyframes dialog-in {
  from {
    opacity: 0;
    transform: scale(0.94) translateY(-6px);
  }
}
</style>
