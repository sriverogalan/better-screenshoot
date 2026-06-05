<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import EditorCanvas from "../components/editor/EditorCanvas.vue";
import EditorStyleBar from "../components/editor/EditorStyleBar.vue";
import EditorToolbar from "../components/editor/EditorToolbar.vue";
import TextAnnotationEditor from "../components/editor/TextAnnotationEditor.vue";
import { useEditorHistory } from "../composables/useEditorHistory";
import { useEditorLayout } from "../composables/useEditorLayout";
import { useEditorShortcuts } from "../composables/useEditorShortcuts";
import { createBlurredRegionDataUrl } from "../lib/editor/blur";
import type { KonvaNode } from "../lib/editor/konva";
import type {
  DraftAnnotation,
  EditorStyle,
  TextEditorState,
  Tool,
} from "../lib/editor/types";
import { DEFAULT_EDITOR_STYLE } from "../lib/editor/types";
import {
  appendTextAnnotation,
  createTextEditorState,
  updateTextEditorState,
} from "../lib/editor/text-editor";
import {
  applyAnnotationDragEnd,
  createAnnotation,
  isAnnotationTooSmall,
  normalizeRect,
  offsetPoints,
  scaleArrowPoints,
  type PointerPosition,
} from "../lib/editor/utils";
import { useCaptureStore } from "../stores/capture";
import { commitPendingText, compositeCaptureExport } from "../lib/editor-export";
import {
  disposeCaptureImage,
  loadCaptureImage,
} from "../lib/load-capture-image";
import type { SavedCapture } from "../lib/tauri";
import { hideEditorWindow } from "../lib/editor-window";
import {
  clearPendingCapture,
  copyImageToClipboard,
  discardCapture,
  saveImageWithDialog,
  takePendingCapture,
} from "../lib/tauri";

const captureStore = useCaptureStore();
const canvasHost = ref<HTMLElement | null>(null);
const canvasRef = ref<InstanceType<typeof EditorCanvas> | null>(null);

const activeTool = ref<Tool>("arrow");
const editorStyle = ref<EditorStyle>({ ...DEFAULT_EDITOR_STYLE });
const selectedId = ref<string | null>(null);
const konvaImage = ref<HTMLImageElement | null>(null);
const imagePreviewSrc = ref<string | null>(null);
const imageNatural = ref({ width: 0, height: 0 });
const drawing = ref(false);
const draft = ref<DraftAnnotation | null>(null);
const textEditor = ref<TextEditorState | null>(null);
const actionBusy = ref(false);
const actionError = ref<string | null>(null);
const imageLoadError = ref<string | null>(null);
let loadGeneration = 0;
let loadTimeoutId: number | undefined;

const FULLSCREEN_LAYOUT_DELAY_MS = 150;
const IMAGE_LOAD_TIMEOUT_MS = 8000;

function isEphemeralCapturePath(filePath: string): boolean {
  return /[/\\]captures[/\\]capture-/.test(filePath);
}

function clearLoadTimeout() {
  if (loadTimeoutId !== undefined) {
    clearTimeout(loadTimeoutId);
    loadTimeoutId = undefined;
  }
}

function scheduleMeasureHost() {
  measureHost();
  requestAnimationFrame(() => {
    measureHost();
    requestAnimationFrame(measureHost);
  });
  window.setTimeout(measureHost, FULLSCREEN_LAYOUT_DELAY_MS);
}

const {
  annotations,
  pushHistory,
  undo,
  redo,
  resetHistory,
  initHistory,
} = useEditorHistory();

const { displayLayout, measureHost } = useEditorLayout(
  canvasHost,
  imageNatural,
  computed(() => captureStore.current?.width),
  computed(() => captureStore.current?.height),
);

const editingAnnotation = computed(() =>
  textEditor.value
    ? annotations.value.find((item) => item.id === textEditor.value?.annotationId)
    : undefined,
);

async function loadCapture(capture: SavedCapture | null) {
  clearLoadTimeout();
  konvaImage.value = null;
  imagePreviewSrc.value = null;
  imageNatural.value = { width: 0, height: 0 };
  imageLoadError.value = null;

  if (!capture) {
    disposeCaptureImage();
    return;
  }

  const generation = ++loadGeneration;
  loadTimeoutId = window.setTimeout(() => {
    if (generation !== loadGeneration) return;
    if (!imagePreviewSrc.value && !imageLoadError.value) {
      imageLoadError.value =
        "No se pudo mostrar la imagen. Comprueba permisos de grabación de pantalla.";
    }
  }, IMAGE_LOAD_TIMEOUT_MS);

  try {
    const loaded = await loadCaptureImage(capture);
    if (generation !== loadGeneration) return;

    clearLoadTimeout();
    konvaImage.value = loaded.element;
    imagePreviewSrc.value = loaded.dataUrl;
    imageNatural.value = {
      width: loaded.element.naturalWidth,
      height: loaded.element.naturalHeight,
    };
    await nextTick();
    scheduleMeasureHost();
  } catch (error) {
    if (generation !== loadGeneration) return;
    clearLoadTimeout();
    imageLoadError.value =
      error instanceof Error ? error.message : "Error al cargar la imagen";
  }
}

watch(
  () => captureStore.current,
  (capture) => {
    void loadCapture(capture);
  },
  { immediate: true },
);

function findAnnotation(id: string) {
  return annotations.value.find((item) => item.id === id);
}

function applyStyleToSelection() {
  if (!selectedId.value) return;
  const annotation = findAnnotation(selectedId.value);
  if (!annotation) return;

  annotation.stroke = editorStyle.value.stroke;
  annotation.strokeWidth = editorStyle.value.strokeWidth;
  if (annotation.tool === "text") {
    annotation.fontSize = editorStyle.value.fontSize;
  }
  if (annotation.tool === "highlight") {
    annotation.fill = editorStyle.value.stroke;
  }
  pushHistory();
}

function onStyleStroke(value: string) {
  editorStyle.value.stroke = value;
  applyStyleToSelection();
}

function onStyleStrokeWidth(value: number) {
  editorStyle.value.strokeWidth = value;
  applyStyleToSelection();
}

function onStyleFontSize(value: number) {
  editorStyle.value.fontSize = value;
  applyStyleToSelection();
}

function setTool(tool: Tool) {
  activeTool.value = tool;
  if (tool !== "select") {
    selectedId.value = null;
  }
}

async function openTextEditor(annotationId: string) {
  const annotation = findAnnotation(annotationId);
  if (!annotation || annotation.tool !== "text") return;

  textEditor.value = createTextEditorState(annotationId, annotation.text);
}

async function placeTextAt(pos: PointerPosition) {
  const { annotations: nextAnnotations, annotation } = appendTextAnnotation(
    annotations.value,
    pos,
    editorStyle.value,
  );
  annotations.value = nextAnnotations;
  await nextTick();
  await openTextEditor(annotation.id);
}

function onTextEditorUpdate(value: string) {
  textEditor.value = updateTextEditorState(textEditor.value, value);
}

async function onImagePointerDown(pos: PointerPosition) {
  if (textEditor.value) return;
  if (activeTool.value !== "text") return;
  await placeTextAt(pos);
}

function cancelTextEdit() {
  const editor = textEditor.value;
  if (!editor) return;

  const annotation = findAnnotation(editor.annotationId);
  if (annotation && annotation.text.trim() === "") {
    annotations.value = annotations.value.filter(
      (item) => item.id !== editor.annotationId,
    );
  }

  textEditor.value = null;
}

function commitTextEdit() {
  const editor = textEditor.value;
  if (!editor) return;

  const result = commitPendingText(editor, annotations.value);
  annotations.value = result.lines;
  if (result.historyChanged) {
    pushHistory();
  }

  textEditor.value = null;
}

function deleteSelected() {
  if (textEditor.value) return;
  if (!selectedId.value) return;

  annotations.value = annotations.value.filter(
    (item) => item.id !== selectedId.value,
  );
  selectedId.value = null;
  pushHistory();
}

function createDraftAt(pos: { x: number; y: number }): DraftAnnotation {
  const base = {
    points: [] as number[],
    x: pos.x,
    y: pos.y,
    width: 0,
    height: 0,
    stroke: editorStyle.value.stroke,
    strokeWidth: editorStyle.value.strokeWidth,
    fontSize: editorStyle.value.fontSize,
    fill: editorStyle.value.stroke,
  };

  if (activeTool.value === "arrow") {
    return {
      ...base,
      tool: "arrow",
      points: [pos.x, pos.y, pos.x, pos.y],
    };
  }

  if (activeTool.value === "pen") {
    return {
      ...base,
      tool: "pen",
      points: [pos.x, pos.y],
    };
  }

  return {
    ...base,
    tool: activeTool.value as DraftAnnotation["tool"],
  };
}

function onStageMouseDown(event: { target: KonvaNode }) {
  if (textEditor.value) return;

  if (activeTool.value === "text" && event.target.getClassName() === "Text") {
    const annotationId = event.target.name();
    if (annotationId) {
      void openTextEditor(annotationId);
      return;
    }
  }

  if (activeTool.value === "select") {
    const className = event.target.getClassName();
    const isShape = !["Stage", "Layer", "Group", "Image"].includes(className);
    if (!isShape) {
      selectedId.value = null;
    }
    return;
  }

  if (activeTool.value === "text") return;

  const pos = usePointerFromEvent(event.target);
  if (!pos) return;

  drawing.value = true;
  draft.value = createDraftAt(pos);
}

function usePointerFromEvent(target: KonvaNode) {
  const stage = target.getStage();
  const pointer = stage.getPointerPosition();
  if (!pointer) return null;
  const layout = displayLayout.value;
  const x = (pointer.x - layout.offsetX) / layout.scale;
  const y = (pointer.y - layout.offsetY) / layout.scale;
  if (x < 0 || y < 0 || x > layout.imageWidth || y > layout.imageHeight) {
    return null;
  }
  return { x, y };
}

function onStageMouseMove(event: { target: KonvaNode }) {
  if (!drawing.value || !draft.value) return;
  const pos = usePointerFromEvent(event.target);
  if (!pos) return;

  if (draft.value.tool === "arrow") {
    draft.value.points = [
      draft.value.points[0],
      draft.value.points[1],
      pos.x,
      pos.y,
    ];
  } else if (draft.value.tool === "pen") {
    draft.value.points = [...draft.value.points, pos.x, pos.y];
  } else if (
    draft.value.tool === "rect" ||
    draft.value.tool === "highlight" ||
    draft.value.tool === "blur"
  ) {
    draft.value.width = pos.x - draft.value.x;
    draft.value.height = pos.y - draft.value.y;
  }
}

async function finalizeActiveDraft(): Promise<boolean> {
  if (!draft.value) return false;

  const currentDraft = draft.value;
  drawing.value = false;
  draft.value = null;

  if (isAnnotationTooSmall(currentDraft)) return false;

  if (currentDraft.tool === "blur" && konvaImage.value) {
    const dataUrl = await createBlurredRegionDataUrl(
      konvaImage.value,
      currentDraft.x,
      currentDraft.y,
      currentDraft.width,
      currentDraft.height,
    );
    if (!dataUrl) return false;

    const rect = normalizeRect(
      currentDraft.x,
      currentDraft.y,
      currentDraft.width,
      currentDraft.height,
    );
    annotations.value.push(
      createAnnotation(
        {
          tool: "blur",
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
          blurImageDataUrl: dataUrl,
        },
        editorStyle.value,
      ),
    );
    pushHistory();
    return true;
  }

  if (
    currentDraft.tool === "rect" ||
    currentDraft.tool === "highlight"
  ) {
    const rect = normalizeRect(
      currentDraft.x,
      currentDraft.y,
      currentDraft.width,
      currentDraft.height,
    );
    annotations.value.push(
      createAnnotation(
        {
          ...currentDraft,
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
        },
        editorStyle.value,
      ),
    );
  } else {
    annotations.value.push(
      createAnnotation(currentDraft, editorStyle.value),
    );
  }
  pushHistory();
  return true;
}

async function onStageMouseUp() {
  if (!drawing.value || !draft.value) return;
  await finalizeActiveDraft();
}

async function prepareForExport() {
  if (textEditor.value) {
    const result = commitPendingText(textEditor.value, annotations.value);
    annotations.value = result.lines;
    if (result.historyChanged) {
      pushHistory();
    }
    textEditor.value = null;
  }

  if (draft.value) {
    await finalizeActiveDraft();
  }

  await nextTick();
}

function onDragEnd(id: string, node: KonvaNode) {
  const annotation = findAnnotation(id);
  if (!annotation) return;

  const { changed } = applyAnnotationDragEnd(
    annotation,
    node.x(),
    node.y(),
  );
  if (!changed) return;

  node.x(0);
  node.y(0);
  pushHistory();
}

function onTransformEnd(id: string, node: KonvaNode) {
  const annotation = findAnnotation(id);
  if (!annotation) return;

  const scaleX = node.scaleX();
  const scaleY = node.scaleY();
  const nodeX = node.x();
  const nodeY = node.y();

  if (annotation.tool === "arrow") {
    annotation.points = scaleArrowPoints(annotation.points, scaleX, scaleY);
    if (nodeX !== 0 || nodeY !== 0) {
      annotation.points = offsetPoints(annotation.points, nodeX, nodeY);
    }
  } else if (
    annotation.tool === "rect" ||
    annotation.tool === "highlight" ||
    annotation.tool === "blur"
  ) {
    annotation.x = nodeX;
    annotation.y = nodeY;
    annotation.width = node.width() * scaleX;
    annotation.height = node.height() * scaleY;
  }

  node.scaleX(1);
  node.scaleY(1);
  node.x(0);
  node.y(0);
  if (
    annotation.tool === "rect" ||
    annotation.tool === "highlight"
  ) {
    const rect = normalizeRect(
      annotation.x,
      annotation.y,
      annotation.width,
      annotation.height,
    );
    annotation.x = rect.x;
    annotation.y = rect.y;
    annotation.width = rect.width;
    annotation.height = rect.height;
  }

  pushHistory();
}

function onSelectAnnotation(id: string | null) {
  selectedId.value = id;
  if (!id) return;

  const annotation = findAnnotation(id);
  if (!annotation) return;

  editorStyle.value = {
    stroke: annotation.stroke,
    strokeWidth: annotation.strokeWidth,
    fontSize: annotation.fontSize,
  };
}

async function exportPngBase64(): Promise<string> {
  const baseImage = konvaImage.value;
  const annotationLayer = canvasRef.value?.getAnnotationLayer() ?? null;
  if (!baseImage || !annotationLayer) {
    throw new Error("La imagen aún no está lista para exportar");
  }

  return compositeCaptureExport(
    baseImage,
    annotationLayer,
    displayLayout.value,
    imageNatural.value.width,
    imageNatural.value.height,
  );
}

async function applyIncomingCapture(capture: SavedCapture) {
  const isSameId = captureStore.current?.id === capture.id;

  if (isSameId) {
    await clearPendingCapture();
    if (imagePreviewSrc.value) {
      scheduleMeasureHost();
      return;
    }
    void loadCapture(capture);
    scheduleMeasureHost();
    return;
  }

  textEditor.value = null;
  selectedId.value = null;
  resetHistory();
  captureStore.setCapture(capture);
  initHistory();
  await clearPendingCapture();
  await nextTick();
  scheduleMeasureHost();
}

async function hydratePendingCapture() {
  const pending = await takePendingCapture();
  if (pending) {
    await applyIncomingCapture(pending);
  }
}

async function closeEditor() {
  await hideEditorWindow();
  textEditor.value = null;
  selectedId.value = null;
  captureStore.clear();
  await clearPendingCapture();
}

async function runEditorAction(action: "discard" | "save") {
  if (actionBusy.value || !konvaImage.value) return;

  actionBusy.value = true;
  actionError.value = null;

  try {
    await prepareForExport();
    const png = await exportPngBase64();
    await copyImageToClipboard(png);

    const pending = captureStore.current;

    if (action === "save") {
      const saved = await saveImageWithDialog(png);
      if (!saved) return;
      if (pending?.id && pending.file_path && isEphemeralCapturePath(pending.file_path)) {
        await discardCapture(pending.id, pending.file_path);
      }
    } else if (pending?.id && pending.file_path) {
      await discardCapture(pending.id, pending.file_path);
    }

    await closeEditor();
  } catch (err) {
    actionError.value =
      err instanceof Error ? err.message : "No se pudo completar la acción";
  } finally {
    actionBusy.value = false;
  }
}

function handleCopyAndDiscard() {
  void runEditorAction("discard");
}

function handleCopyAndSave() {
  void runEditorAction("save");
}

function handleUndo() {
  if (undo()) {
    textEditor.value = null;
    selectedId.value = null;
  }
}

function handleRedo() {
  if (redo()) {
    textEditor.value = null;
    selectedId.value = null;
  }
}

useEditorShortcuts({
  undo: handleUndo,
  redo: handleRedo,
  deleteSelected,
  deselect: () => {
    selectedId.value = null;
    if (textEditor.value) cancelTextEdit();
  },
  setTool,
  isTextEditing: () => textEditor.value !== null,
});

let unlisten: UnlistenFn | undefined;
let resizeObserver: ResizeObserver | undefined;

onMounted(async () => {
  const win = getCurrentWindow();
  if (win.label === "editor") {
    await win.onCloseRequested(async (event) => {
      event.preventDefault();
      await closeEditor();
    });

    await hydratePendingCapture();

    await win.onFocusChanged(async ({ payload: focused }) => {
      if (!focused) return;
      await hydratePendingCapture();
      scheduleMeasureHost();
    });
  }

  measureHost();
  window.addEventListener("resize", measureHost);

  if (canvasHost.value) {
    resizeObserver = new ResizeObserver(() => {
      measureHost();
    });
    resizeObserver.observe(canvasHost.value);
  }

  unlisten = await listen<SavedCapture>("capture-complete", (event) => {
    void applyIncomingCapture(event.payload);
  });

  if (captureStore.current) {
    initHistory();
  }
});

onUnmounted(() => {
  clearLoadTimeout();
  window.removeEventListener("resize", measureHost);
  resizeObserver?.disconnect();
  unlisten?.();
  disposeCaptureImage();
});
</script>

<template>
  <div class="grid h-full min-h-0 w-full grid-rows-[auto_auto_1fr] bg-surface">
    <EditorToolbar
      :active-tool="activeTool"
      :action-busy="actionBusy"
      :action-error="actionError"
      :can-export="!!imagePreviewSrc"
      :image-width="displayLayout.imageWidth"
      :image-height="displayLayout.imageHeight"
      :zoom-percent="displayLayout.zoomPercent"
      :has-capture="!!captureStore.current"
      @update:active-tool="setTool"
      @undo="handleUndo"
      @redo="handleRedo"
      @copy-and-discard="handleCopyAndDiscard"
      @copy-and-save="handleCopyAndSave"
    />

    <EditorStyleBar
      :style="editorStyle"
      @update:stroke="onStyleStroke"
      @update:stroke-width="onStyleStrokeWidth"
      @update:font-size="onStyleFontSize"
    />

    <div ref="canvasHost" class="relative min-h-0 h-full">
      <EditorCanvas
        ref="canvasRef"
        :image-preview-src="imagePreviewSrc"
        :konva-image="konvaImage"
        :layout="displayLayout"
        :annotations="annotations"
        :draft="draft"
        :active-tool="activeTool"
        :selected-id="selectedId"
        :text-editor="textEditor"
        :has-capture="!!captureStore.current"
        :image-load-error="imageLoadError"
        @stage-mouse-down="onStageMouseDown"
        @stage-mouse-move="onStageMouseMove"
        @stage-mouse-up="onStageMouseUp"
        @image-pointer-down="onImagePointerDown"
        @select-annotation="onSelectAnnotation"
        @drag-end="onDragEnd"
        @transform-end="onTransformEnd"
        @open-text-editor="openTextEditor"
      />

      <div
        v-if="textEditor && editingAnnotation && imagePreviewSrc"
        class="pointer-events-none absolute inset-0 z-30"
      >
        <TextAnnotationEditor
          :editor="textEditor"
          :annotation="editingAnnotation"
          :layout="displayLayout"
          @update="onTextEditorUpdate"
          @commit="commitTextEdit"
          @cancel="cancelTextEdit"
        />
      </div>
    </div>
  </div>
</template>
