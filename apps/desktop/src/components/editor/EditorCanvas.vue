<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import type {
  KonvaNode,
  KonvaStageRef,
  KonvaTransformerRef,
} from "../../lib/editor/konva";
import type {
  Annotation,
  DisplayLayout,
  DraftAnnotation,
  TextEditorState,
  Tool,
} from "../../lib/editor/types";
import {
  getHighlightOpacity,
  getImagePointer,
  normalizeRect,
} from "../../lib/editor/utils";
import type { PointerPosition } from "../../lib/editor/utils";

const { t } = useI18n();

const props = defineProps<{
  imagePreviewSrc: string | null;
  konvaImage: HTMLImageElement | null;
  layout: DisplayLayout;
  annotations: Annotation[];
  draft: DraftAnnotation | null;
  activeTool: Tool;
  selectedId: string | null;
  textEditor: TextEditorState | null;
  hasCapture: boolean;
  imageLoadError: string | null;
}>();

const emit = defineEmits<{
  stageMouseDown: [event: { target: KonvaNode }];
  stageMouseMove: [event: { target: KonvaNode }];
  stageMouseUp: [];
  imagePointerDown: [pos: PointerPosition];
  selectAnnotation: [id: string | null];
  dragEnd: [id: string, node: KonvaNode];
  transformEnd: [id: string, node: KonvaNode];
  openTextEditor: [id: string];
}>();

const stageRef = ref<KonvaStageRef | null>(null);
const annotationLayerRef = ref<{ getNode: () => AnnotationLayerNode | null } | null>(
  null,
);
const transformerRef = ref<KonvaTransformerRef | null>(null);

type AnnotationLayerNode = {
  toDataURL: (config: {
    pixelRatio: number;
    x: number;
    y: number;
    width: number;
    height: number;
  }) => string;
  batchDraw: () => void;
};

const blurImages = ref<Map<string, HTMLImageElement>>(new Map());

const arrows = computed(() =>
  props.annotations.filter((item) => item.tool === "arrow"),
);
const rects = computed(() =>
  props.annotations.filter((item) => item.tool === "rect"),
);
const highlights = computed(() =>
  props.annotations.filter((item) => item.tool === "highlight"),
);
const pens = computed(() =>
  props.annotations.filter((item) => item.tool === "pen"),
);
const texts = computed(() =>
  props.annotations.filter((item) => item.tool === "text"),
);
const blurs = computed(() =>
  props.annotations.filter((item) => item.tool === "blur"),
);

function isDraggable(id: string) {
  return props.activeTool === "select" && props.textEditor?.annotationId !== id;
}

function shapeListening() {
  return true;
}

const imageHitListening = computed(() => props.activeTool !== "select");

function onImageHitMouseDown(event: { target: KonvaNode }) {
  const stage = event.target.getStage();
  const pointer = stage.getPointerPosition();
  if (!pointer) return;

  const pos = getImagePointer(pointer.x, pointer.y, props.layout);
  if (!pos) return;

  emit("imagePointerDown", pos);
}

function getBlurImage(annotation: Annotation): HTMLImageElement | null {
  if (!annotation.blurImageDataUrl) return null;
  const cached = blurImages.value.get(annotation.id);
  if (cached) return cached;
  const img = new Image();
  img.src = annotation.blurImageDataUrl;
  blurImages.value.set(annotation.id, img);
  return img;
}

watch(
  () => props.annotations.filter((a) => a.tool === "blur"),
  (blurAnnotations) => {
    const ids = new Set(blurAnnotations.map((a) => a.id));
    for (const id of blurImages.value.keys()) {
      if (!ids.has(id)) blurImages.value.delete(id);
    }
    for (const annotation of blurAnnotations) {
      if (annotation.blurImageDataUrl && !blurImages.value.has(annotation.id)) {
        const img = new Image();
        img.src = annotation.blurImageDataUrl;
        blurImages.value.set(annotation.id, img);
      }
    }
  },
  { deep: true },
);

function updateTransformer() {
  const transformer = transformerRef.value?.getNode() as {
    nodes: (nodes: KonvaNode[]) => void;
    resizeEnabled: (value: boolean) => void;
    rotateEnabled: (value: boolean) => void;
    enabledAnchors: (anchors: string[]) => void;
    getLayer: () => { batchDraw: () => void } | null;
  } | undefined;
  const stage = stageRef.value?.getStage();
  if (!transformer || !stage) return;

  if (!props.selectedId) {
    transformer.nodes([]);
    transformer.getLayer()?.batchDraw();
    return;
  }

  const selected = props.annotations.find((a) => a.id === props.selectedId);
  if (!selected || selected.tool === "pen") {
    transformer.nodes([]);
    transformer.getLayer()?.batchDraw();
    return;
  }

  const node = stage.findOne(`#${props.selectedId}`);
  if (!node) {
    transformer.nodes([]);
    transformer.getLayer()?.batchDraw();
    return;
  }

  const resizeEnabled = selected.tool !== "text";
  transformer.resizeEnabled(resizeEnabled);
  transformer.rotateEnabled(false);
  transformer.enabledAnchors(
    selected.tool === "text"
      ? []
      : [
          "top-left",
          "top-right",
          "bottom-left",
          "bottom-right",
          "middle-left",
          "middle-right",
          "top-center",
          "bottom-center",
        ],
  );
  transformer.nodes([node]);
  transformer.getLayer()?.batchDraw();
}

watch(
  () => [props.selectedId, props.annotations, props.activeTool],
  () => {
    requestAnimationFrame(updateTransformer);
  },
  { deep: true },
);

watch(
  () => props.imagePreviewSrc,
  (src) => {
    if (!src) return;
    requestAnimationFrame(() => {
      const layer = annotationLayerRef.value?.getNode();
      layer?.batchDraw();
    });
  },
);

function onShapeClick(id: string, event: { cancelBubble: boolean }) {
  if (props.activeTool !== "select") return;
  event.cancelBubble = true;
  emit("selectAnnotation", id);
}

function getStage() {
  return stageRef.value?.getStage() ?? null;
}

function getAnnotationLayer() {
  return annotationLayerRef.value?.getNode() ?? null;
}

defineExpose({
  getStage,
  getAnnotationLayer,
  stageRef,
});
</script>

<template>
  <div class="relative h-full min-h-0 overflow-hidden bg-[#0a0c10]">
    <img
      v-if="imagePreviewSrc"
      class="pointer-events-none absolute select-none"
      :src="imagePreviewSrc"
      :style="{
        left: `${layout.offsetX}px`,
        top: `${layout.offsetY}px`,
        width: `${layout.displayW}px`,
        height: `${layout.displayH}px`,
      }"
      draggable="false"
      alt=""
    />

    <v-stage
      v-if="imagePreviewSrc && layout.stageWidth > 0"
      ref="stageRef"
      class="absolute inset-0"
      :config="{
        width: layout.stageWidth,
        height: layout.stageHeight,
      }"
      @mousedown="emit('stageMouseDown', $event)"
      @mousemove="emit('stageMouseMove', $event)"
      @mouseup="emit('stageMouseUp')"
    >
      <v-layer :config="{ listening: false }">
        <v-rect
          :config="{
            x: layout.offsetX,
            y: layout.offsetY,
            width: layout.displayW,
            height: layout.displayH,
            fill: 'transparent',
            stroke: '#2a2f3a',
            strokeWidth: 1,
            shadowColor: '#000000',
            shadowBlur: 24,
            shadowOpacity: 0.45,
            listening: false,
          }"
        />
      </v-layer>

      <v-layer ref="annotationLayerRef">
        <v-group
          :config="{
            x: layout.offsetX,
            y: layout.offsetY,
            scaleX: layout.scale,
            scaleY: layout.scale,
          }"
        >
          <v-rect
            :config="{
              x: 0,
              y: 0,
              width: layout.imageWidth,
              height: layout.imageHeight,
              fill: 'transparent',
              listening: imageHitListening,
            }"
            @mousedown="onImageHitMouseDown"
          />

          <v-image
            v-for="item in blurs"
            :key="item.id"
            :config="{
              id: item.id,
              name: item.id,
              image: getBlurImage(item),
              x: normalizeRect(item.x, item.y, item.width, item.height).x,
              y: normalizeRect(item.x, item.y, item.width, item.height).y,
              width: normalizeRect(item.x, item.y, item.width, item.height).width,
              height: normalizeRect(item.x, item.y, item.width, item.height).height,
              draggable: isDraggable(item.id),
              listening: shapeListening(),
            }"
            @click="onShapeClick(item.id, $event)"
            @dragend="emit('dragEnd', item.id, $event.target)"
            @transformend="emit('transformEnd', item.id, $event.target)"
          />

          <v-rect
            v-for="item in highlights"
            :key="item.id"
            :config="{
              id: item.id,
              name: item.id,
              x: normalizeRect(item.x, item.y, item.width, item.height).x,
              y: normalizeRect(item.x, item.y, item.width, item.height).y,
              width: normalizeRect(item.x, item.y, item.width, item.height).width,
              height: normalizeRect(item.x, item.y, item.width, item.height).height,
              fill: item.fill ?? item.stroke,
              opacity: getHighlightOpacity(),
              stroke: item.stroke,
              strokeWidth: 1,
              draggable: isDraggable(item.id),
              listening: shapeListening(),
            }"
            @click="onShapeClick(item.id, $event)"
            @dragend="emit('dragEnd', item.id, $event.target)"
            @transformend="emit('transformEnd', item.id, $event.target)"
          />

          <v-line
            v-for="item in pens"
            :key="item.id"
            :config="{
              id: item.id,
              name: item.id,
              points: item.points,
              stroke: item.stroke,
              strokeWidth: item.strokeWidth,
              tension: 0.5,
              lineCap: 'round',
              lineJoin: 'round',
              draggable: isDraggable(item.id),
              listening: shapeListening(),
            }"
            @click="onShapeClick(item.id, $event)"
            @dragend="emit('dragEnd', item.id, $event.target)"
          />

          <v-arrow
            v-for="item in arrows"
            :key="item.id"
            :config="{
              id: item.id,
              name: item.id,
              points: item.points,
              stroke: item.stroke,
              strokeWidth: item.strokeWidth,
              pointerLength: 10,
              pointerWidth: 10,
              draggable: isDraggable(item.id),
              listening: shapeListening(),
            }"
            @click="onShapeClick(item.id, $event)"
            @dragend="emit('dragEnd', item.id, $event.target)"
            @transformend="emit('transformEnd', item.id, $event.target)"
          />

          <v-rect
            v-for="item in rects"
            :key="item.id"
            :config="{
              id: item.id,
              name: item.id,
              x: normalizeRect(item.x, item.y, item.width, item.height).x,
              y: normalizeRect(item.x, item.y, item.width, item.height).y,
              width: normalizeRect(item.x, item.y, item.width, item.height).width,
              height: normalizeRect(item.x, item.y, item.width, item.height).height,
              stroke: item.stroke,
              strokeWidth: item.strokeWidth,
              draggable: isDraggable(item.id),
              listening: shapeListening(),
            }"
            @click="onShapeClick(item.id, $event)"
            @dragend="emit('dragEnd', item.id, $event.target)"
            @transformend="emit('transformEnd', item.id, $event.target)"
          />

          <v-text
            v-for="item in texts"
            :key="item.id"
            :config="{
              id: item.id,
              name: item.id,
              x: item.x,
              y: item.y,
              text: item.text || ' ',
              fontSize: item.fontSize,
              fill: item.stroke,
              visible: textEditor?.annotationId !== item.id,
              shadowColor: '#000000',
              shadowBlur: 6,
              shadowOffset: { x: 1, y: 1 },
              shadowOpacity: 0.85,
              draggable: isDraggable(item.id),
              listening: shapeListening(),
            }"
            @click="onShapeClick(item.id, $event)"
            @dblclick="emit('openTextEditor', item.id)"
            @dragend="emit('dragEnd', item.id, $event.target)"
          />

          <v-arrow
            v-if="draft?.tool === 'arrow'"
            :config="{
              points: draft.points,
              stroke: draft.stroke,
              strokeWidth: draft.strokeWidth,
              pointerLength: 10,
              pointerWidth: 10,
            }"
          />
          <v-rect
            v-if="
              draft &&
              (draft.tool === 'rect' ||
                draft.tool === 'highlight' ||
                draft.tool === 'blur')
            "
            :config="{
              x: normalizeRect(draft.x, draft.y, draft.width, draft.height).x,
              y: normalizeRect(draft.x, draft.y, draft.width, draft.height).y,
              width: normalizeRect(draft.x, draft.y, draft.width, draft.height).width,
              height: normalizeRect(draft.x, draft.y, draft.width, draft.height).height,
              stroke: draft.stroke,
              strokeWidth: draft.strokeWidth,
              fill:
                draft.tool === 'highlight'
                  ? draft.fill ?? draft.stroke
                  : undefined,
              opacity: draft.tool === 'highlight' ? getHighlightOpacity() : 1,
              dash: draft.tool === 'blur' ? [6, 4] : undefined,
            }"
          />
          <v-line
            v-if="draft?.tool === 'pen'"
            :config="{
              points: draft.points,
              stroke: draft.stroke,
              strokeWidth: draft.strokeWidth,
              tension: 0.5,
              lineCap: 'round',
              lineJoin: 'round',
            }"
          />
        </v-group>

        <v-transformer ref="transformerRef" />
      </v-layer>
    </v-stage>

    <p
      v-if="imageLoadError"
      class="absolute inset-0 flex items-center justify-center px-6 text-center text-sm text-danger"
      role="alert"
    >
      {{ imageLoadError }}
    </p>
    <p
      v-else-if="!imagePreviewSrc && hasCapture"
      class="absolute inset-0 flex items-center justify-center text-sm text-fg-muted"
    >
      {{ t("editor.loadingImage") }}
    </p>
    <p
      v-else-if="!hasCapture"
      class="absolute inset-0 flex items-center justify-center text-sm text-fg-muted"
    >
      {{ t("editor.empty") }}
    </p>
  </div>
</template>
