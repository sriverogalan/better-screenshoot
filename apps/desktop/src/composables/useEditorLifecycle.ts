import { nextTick, ref, type Ref } from "vue"
import { useI18n } from "vue-i18n"
import { disposeCaptureImage, loadCaptureImage } from "../lib/load-capture-image"
import type { SavedCapture } from "../lib/tauri"

const IMAGE_LOAD_TIMEOUT_MS = 8000

export interface UseEditorLifecycleOptions {
  onLoaded: () => void
}

export interface UseEditorLifecycleReturn {
  konvaImage: Ref<HTMLImageElement | null>
  imagePreviewSrc: Ref<string | null>
  imageNatural: Ref<{ width: number; height: number }>
  imageLoadError: Ref<string | null>
  loadCapture: (capture: SavedCapture | null) => Promise<void>
  cleanup: () => void
}

export function useEditorLifecycle(options: UseEditorLifecycleOptions): UseEditorLifecycleReturn {
  const { t } = useI18n()

  const konvaImage = ref<HTMLImageElement | null>(null)
  const imagePreviewSrc = ref<string | null>(null)
  const imageNatural = ref({ width: 0, height: 0 })
  const imageLoadError = ref<string | null>(null)

  // Plain number — NOT a ref. Internal cancellation token, not reactive state.
  let loadGeneration = 0
  let loadTimeoutId: number | undefined

  function clearLoadTimeout(): void {
    if (loadTimeoutId !== undefined) {
      clearTimeout(loadTimeoutId)
      loadTimeoutId = undefined
    }
  }

  async function loadCapture(capture: SavedCapture | null): Promise<void> {
    clearLoadTimeout()
    konvaImage.value = null
    imagePreviewSrc.value = null
    imageNatural.value = { width: 0, height: 0 }
    imageLoadError.value = null

    if (!capture) {
      disposeCaptureImage()
      return
    }

    const generation = ++loadGeneration

    loadTimeoutId = window.setTimeout(() => {
      if (generation !== loadGeneration) return
      if (!imagePreviewSrc.value && !imageLoadError.value) {
        imageLoadError.value = t("errors.imageDisplayFailed")
      }
    }, IMAGE_LOAD_TIMEOUT_MS)

    try {
      const loaded = await loadCaptureImage(capture)
      if (generation !== loadGeneration) return

      clearLoadTimeout()
      konvaImage.value = loaded.element
      imagePreviewSrc.value = loaded.dataUrl
      imageNatural.value = {
        width: loaded.element.naturalWidth,
        height: loaded.element.naturalHeight,
      }
      await nextTick()
      if (generation !== loadGeneration) return
      options.onLoaded()
    } catch (error) {
      if (generation !== loadGeneration) return
      clearLoadTimeout()
      imageLoadError.value =
        error instanceof Error ? error.message : t("errors.imageLoadFailed")
    }
  }

  function cleanup(): void {
    clearLoadTimeout()
    disposeCaptureImage()
  }

  return { konvaImage, imagePreviewSrc, imageNatural, imageLoadError, loadCapture, cleanup }
}
