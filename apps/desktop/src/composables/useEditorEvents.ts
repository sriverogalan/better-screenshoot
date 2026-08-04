import { listen, type UnlistenFn } from "@tauri-apps/api/event"
import type { Ref } from "vue"
import type { SavedCapture } from "../lib/tauri"

export interface UseEditorEventsOptions {
  canvasHost: Ref<HTMLElement | null>
  measureHost: () => void
  onCaptureComplete: (capture: SavedCapture) => void
  onEditorPresented: () => void
}

export interface UseEditorEventsReturn {
  setup: () => Promise<void>
  cleanup: () => void
}

export function useEditorEvents(options: UseEditorEventsOptions): UseEditorEventsReturn {
  let unlistenCaptureComplete: UnlistenFn | undefined
  let unlistenEditorPresented: UnlistenFn | undefined
  let resizeObserver: ResizeObserver | undefined

  async function setup(): Promise<void> {
    window.addEventListener("resize", options.measureHost)

    if (options.canvasHost.value) {
      resizeObserver = new ResizeObserver(() => options.measureHost())
      resizeObserver.observe(options.canvasHost.value)
    }

    unlistenCaptureComplete = await listen<SavedCapture>("capture-complete", (event) => {
      options.onCaptureComplete(event.payload)
    })

    unlistenEditorPresented = await listen("editor-presented", () => {
      options.onEditorPresented()
    })
  }

  function cleanup(): void {
    window.removeEventListener("resize", options.measureHost)
    resizeObserver?.disconnect()
    unlistenCaptureComplete?.()
    unlistenEditorPresented?.()
  }

  return { setup, cleanup }
}
