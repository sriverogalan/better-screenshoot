<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import type { Annotation, DisplayLayout, TextEditorState } from "../../lib/editor/types";
import { measureTextBlock } from "../../lib/editor/utils";

const props = defineProps<{
  editor: TextEditorState;
  annotation: Annotation | undefined;
  layout: DisplayLayout;
}>();

const emit = defineEmits<{
  update: [value: string];
  commit: [];
  cancel: [];
}>();

const textareaRef = ref<HTMLTextAreaElement | null>(null);
const ignoreBlur = ref(true);

const editorStyle = computed(() => {
  if (!props.annotation) {
    return { display: "none" };
  }

  const { scale, offsetX, offsetY } = props.layout;
  const fontSize = props.annotation.fontSize * scale;
  const measured = measureTextBlock(
    props.editor.value || " ",
    props.annotation.fontSize,
  );

  return {
    left: `${offsetX + props.annotation.x * scale}px`,
    top: `${offsetY + props.annotation.y * scale}px`,
    fontSize: `${fontSize}px`,
    lineHeight: "1.25",
    minWidth: `${measured.width * scale}px`,
    width: `${Math.max(measured.width, 120) * scale}px`,
    color: props.annotation.stroke,
  };
});

function autoGrow() {
  const el = textareaRef.value;
  if (!el) return;
  el.style.height = "auto";
  el.style.height = `${el.scrollHeight}px`;
}

watch(
  () => props.editor.value,
  async () => {
    await nextTick();
    autoGrow();
  },
  { immediate: true },
);

watch(textareaRef, async (el) => {
  if (!el) return;
  ignoreBlur.value = true;
  await nextTick();
  el.focus();
  el.select();
  autoGrow();
  requestAnimationFrame(() => {
    ignoreBlur.value = false;
  });
});

onMounted(() => {
  requestAnimationFrame(() => {
    ignoreBlur.value = false;
  });
});

function onBlur() {
  if (ignoreBlur.value) return;
  emit("commit");
}

function onInput(event: Event) {
  const value = (event.target as HTMLTextAreaElement).value;
  emit("update", value);
  autoGrow();
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    emit("cancel");
    return;
  }

  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    textareaRef.value?.blur();
  }
}
</script>

<template>
  <textarea
    ref="textareaRef"
    :value="editor.value"
    rows="1"
    class="pointer-events-auto absolute z-20 resize-none overflow-hidden rounded-md border-2 border-accent bg-black/85 px-2 py-1 shadow-lg outline-none ring-2 ring-accent/30 placeholder:text-white/50"
    :style="editorStyle"
    placeholder="Type here…"
    aria-label="Edit annotation text"
    @input="onInput"
    @blur="onBlur"
    @mousedown.stop
    @keydown="onKeydown"
  />
</template>
