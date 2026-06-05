import { loadHtmlImage } from "./load-capture-image";
import type { Annotation, TextEditorState } from "./editor/types";

export interface CommitPendingTextResult {
  lines: Annotation[];
  historyChanged: boolean;
}

export function commitPendingText(
  editor: TextEditorState,
  lines: Annotation[],
): CommitPendingTextResult {
  const annotation = lines.find((line) => line.id === editor.annotationId);
  const trimmed = editor.value.trim();

  if (!annotation) {
    return { lines, historyChanged: false };
  }

  if (trimmed === "") {
    return {
      lines: lines.filter((line) => line.id !== editor.annotationId),
      historyChanged: false,
    };
  }

  const updated = lines.map((line) =>
    line.id === editor.annotationId ? { ...line, text: trimmed } : line,
  );
  return { lines: updated, historyChanged: true };
}

export interface ExportLayout {
  scale: number;
  offsetX: number;
  offsetY: number;
  displayW: number;
  displayH: number;
}

export interface AnnotationLayer {
  toDataURL: (config: {
    pixelRatio: number;
    x: number;
    y: number;
    width: number;
    height: number;
  }) => string;
}

export async function compositeCaptureExport(
  baseImage: HTMLImageElement,
  annotationLayer: AnnotationLayer,
  layout: ExportLayout,
  nativeWidth: number,
  nativeHeight: number,
): Promise<string> {
  const { scale, offsetX, offsetY, displayW, displayH } = layout;
  const pixelRatio = scale > 0 ? 1 / scale : 1;

  const canvas = document.createElement("canvas");
  canvas.width = nativeWidth;
  canvas.height = nativeHeight;
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("No se pudo preparar la exportación");
  }

  ctx.drawImage(baseImage, 0, 0, nativeWidth, nativeHeight);

  const annotationsDataUrl = annotationLayer.toDataURL({
    pixelRatio,
    x: offsetX,
    y: offsetY,
    width: displayW,
    height: displayH,
  });

  const overlay = await loadHtmlImage(annotationsDataUrl);
  ctx.drawImage(overlay, 0, 0, nativeWidth, nativeHeight);

  return canvas
    .toDataURL("image/png")
    .replace(/^data:image\/png;base64,/, "");
}
