import { computed, ref, type ComputedRef, type Ref } from "vue";
import type { DisplayLayout } from "../lib/editor/types";
import { HEADER_HEIGHT } from "../lib/editor/types";
import type { KonvaStage } from "../lib/editor/konva";
import { computeDisplayLayout, getImagePointer } from "../lib/editor/utils";

export function useEditorLayout(
  canvasHost: Ref<HTMLElement | null>,
  imageNatural: Ref<{ width: number; height: number }>,
  fallbackWidth: Ref<number | undefined> | ComputedRef<number | undefined>,
  fallbackHeight: Ref<number | undefined> | ComputedRef<number | undefined>,
) {
  const hostSize = ref({ width: 0, height: 0 });

  const displayLayout = computed<DisplayLayout>(() => {
    const imageWidth = imageNatural.value.width || fallbackWidth.value || 1;
    const imageHeight = imageNatural.value.height || fallbackHeight.value || 1;
    return computeDisplayLayout(
      imageWidth,
      imageHeight,
      hostSize.value.width,
      hostSize.value.height,
    );
  });

  function measureHost() {
    const el = canvasHost.value;
    if (!el) return;

    const width = el.clientWidth;
    const height = el.clientHeight;
    if (width > 0 && height > 0) {
      hostSize.value = { width, height };
      return;
    }

    hostSize.value = {
      width: Math.max(window.innerWidth, 1),
      height: Math.max(window.innerHeight - HEADER_HEIGHT, 1),
    };
  }

  function pointerFromStage(stage: KonvaStage) {
    const pos = stage.getPointerPosition();
    if (!pos) return null;
    return getImagePointer(pos.x, pos.y, displayLayout.value);
  }

  return {
    hostSize,
    displayLayout,
    measureHost,
    pointerFromStage,
  };
}
