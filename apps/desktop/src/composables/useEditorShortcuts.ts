import { onMounted, onUnmounted } from "vue";
import type { Tool } from "../lib/editor/types";

interface EditorShortcutHandlers {
  undo: () => void;
  redo: () => void;
  deleteSelected: () => void;
  deselect: () => void;
  setTool: (tool: Tool) => void;
  isTextEditing: () => boolean;
}

export function useEditorShortcuts(handlers: EditorShortcutHandlers) {
  function onKeydown(event: KeyboardEvent) {
    const target = event.target;
    const isTextarea =
      target instanceof HTMLTextAreaElement ||
      target instanceof HTMLInputElement;

    if (isTextarea && handlers.isTextEditing()) {
      if (event.key === "Escape") return;
      return;
    }

    const mod = event.metaKey || event.ctrlKey;

    if (mod && event.key === "z" && !event.shiftKey) {
      event.preventDefault();
      handlers.undo();
      return;
    }

    if (mod && (event.key === "Z" || (event.key === "z" && event.shiftKey))) {
      event.preventDefault();
      handlers.redo();
      return;
    }

    if (event.key === "Delete" || event.key === "Backspace") {
      if (isTextarea) return;
      event.preventDefault();
      handlers.deleteSelected();
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      handlers.deselect();
      return;
    }

    if (mod || event.altKey) return;

    const toolKeys: Record<string, Tool> = {
      v: "select",
      a: "arrow",
      r: "rect",
      t: "text",
      h: "highlight",
      p: "pen",
      b: "blur",
    };

    const tool = toolKeys[event.key.toLowerCase()];
    if (tool) {
      event.preventDefault();
      handlers.setTool(tool);
    }
  }

  onMounted(() => {
    window.addEventListener("keydown", onKeydown);
  });

  onUnmounted(() => {
    window.removeEventListener("keydown", onKeydown);
  });
}
