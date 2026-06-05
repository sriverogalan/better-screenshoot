import { ref } from "vue";
import type { Annotation } from "../lib/editor/types";
import { cloneAnnotations } from "../lib/editor/utils";

export function useEditorHistory() {
  const annotations = ref<Annotation[]>([]);
  const history = ref<Annotation[][]>([]);
  const historyIndex = ref(-1);

  function pushHistory() {
    history.value = history.value.slice(0, historyIndex.value + 1);
    history.value.push(cloneAnnotations(annotations.value));
    historyIndex.value = history.value.length - 1;
  }

  function undo(): boolean {
    if (historyIndex.value <= 0) return false;
    historyIndex.value -= 1;
    annotations.value = cloneAnnotations(history.value[historyIndex.value]);
    return true;
  }

  function redo(): boolean {
    if (historyIndex.value >= history.value.length - 1) return false;
    historyIndex.value += 1;
    annotations.value = cloneAnnotations(history.value[historyIndex.value]);
    return true;
  }

  function resetHistory() {
    annotations.value = [];
    history.value = [];
    historyIndex.value = -1;
  }

  function initHistory() {
    pushHistory();
  }

  return {
    annotations,
    history,
    historyIndex,
    pushHistory,
    undo,
    redo,
    resetHistory,
    initHistory,
  };
}
