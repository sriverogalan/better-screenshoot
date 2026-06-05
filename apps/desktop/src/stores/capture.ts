import { defineStore } from "pinia";
import { ref } from "vue";
import type { SavedCapture } from "../lib/tauri";

export const useCaptureStore = defineStore("capture", () => {
  const current = ref<SavedCapture | null>(null);
  const editorBase64 = ref<string | null>(null);

  function setCapture(capture: SavedCapture) {
    current.value = capture;
    editorBase64.value = capture.data_url
      ? capture.data_url.replace(/^data:image\/png;base64,/, "")
      : null;
  }

  function clear() {
    current.value = null;
    editorBase64.value = null;
  }

  return { current, editorBase64, setCapture, clear };
});
