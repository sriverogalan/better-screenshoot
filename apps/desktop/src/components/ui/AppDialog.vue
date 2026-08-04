<script setup lang="ts">
import { nextTick, onUnmounted, ref, useId, watch } from "vue";
import { IconX } from "@tabler/icons-vue";
import { useI18n } from "vue-i18n";

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

const { t } = useI18n();
const closeButtonRef = ref<HTMLButtonElement | null>(null);
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
    closeButtonRef.value?.focus();
  },
);

onUnmounted(() => {
  document.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
      @click.self="onOverlayClick"
    >
      <div
        role="dialog"
        aria-modal="true"
        :aria-labelledby="titleId"
        :aria-describedby="description ? descriptionId : undefined"
        class="w-full max-w-lg rounded-xl border border-sep bg-elev p-5 shadow-window"
      >
        <div class="mb-4 flex items-start justify-between gap-4">
          <div class="min-w-0">
            <h2 :id="titleId" class="text-base font-semibold text-fg">
              {{ title }}
            </h2>
            <p
              v-if="description"
              :id="descriptionId"
              class="mt-1 text-sm text-fg-muted"
            >
              {{ description }}
            </p>
          </div>
          <button
            ref="closeButtonRef"
            type="button"
            class="shrink-0 rounded-lg p-1 text-fg-muted hover:bg-sep hover:text-fg disabled:opacity-50"
            :aria-label="t('common.close')"
            :disabled="busy"
            @click="emit('close')"
          >
            <IconX class="size-5" />
          </button>
        </div>

        <div v-if="$slots.default">
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
  </Teleport>
</template>
