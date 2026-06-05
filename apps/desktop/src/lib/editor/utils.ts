import type {
  Annotation,
  DisplayLayout,
  DraftAnnotation,
  EditorStyle,
} from "./types";
import { HIGHLIGHT_OPACITY, MIN_DRAW_SIZE } from "./types";

export interface PointerPosition {
  x: number;
  y: number;
}

export interface NormalizedRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function computeDisplayLayout(
  imageWidth: number,
  imageHeight: number,
  hostWidth: number,
  hostHeight: number,
): DisplayLayout {
  const availW = Math.max(hostWidth, 1);
  const availH = Math.max(hostHeight, 1);
  const scale = Math.min(availW / imageWidth, availH / imageHeight);
  const displayW = imageWidth * scale;
  const displayH = imageHeight * scale;

  return {
    stageWidth: Math.round(availW),
    stageHeight: Math.round(availH),
    scale,
    offsetX: (availW - displayW) / 2,
    offsetY: (availH - displayH) / 2,
    displayW: Math.round(displayW),
    displayH: Math.round(displayH),
    zoomPercent: Math.round(scale * 100),
    imageWidth,
    imageHeight,
  };
}

export function getImagePointer(
  stageX: number,
  stageY: number,
  layout: DisplayLayout,
): PointerPosition | null {
  if (layout.scale <= 0) return null;

  const x = (stageX - layout.offsetX) / layout.scale;
  const y = (stageY - layout.offsetY) / layout.scale;

  if (x < 0 || y < 0 || x > layout.imageWidth || y > layout.imageHeight) {
    return null;
  }

  return { x, y };
}

export function normalizeRect(
  x: number,
  y: number,
  width: number,
  height: number,
): NormalizedRect {
  return {
    x: width < 0 ? x + width : x,
    y: height < 0 ? y + height : y,
    width: Math.abs(width),
    height: Math.abs(height),
  };
}

export function isAnnotationTooSmall(annotation: DraftAnnotation): boolean {
  if (annotation.tool === "arrow") {
    const [x1, y1, x2, y2] = annotation.points;
    const dx = Math.abs(x2 - x1);
    const dy = Math.abs(y2 - y1);
    return dx < MIN_DRAW_SIZE && dy < MIN_DRAW_SIZE;
  }

  if (
    annotation.tool === "rect" ||
    annotation.tool === "highlight" ||
    annotation.tool === "blur"
  ) {
    return (
      Math.abs(annotation.width) < MIN_DRAW_SIZE &&
      Math.abs(annotation.height) < MIN_DRAW_SIZE
    );
  }

  if (annotation.tool === "pen") {
    if (annotation.points.length < 4) return true;
    let minX = annotation.points[0];
    let maxX = annotation.points[0];
    let minY = annotation.points[1];
    let maxY = annotation.points[1];
    for (let i = 2; i < annotation.points.length; i += 2) {
      minX = Math.min(minX, annotation.points[i]);
      maxX = Math.max(maxX, annotation.points[i]);
      minY = Math.min(minY, annotation.points[i + 1]);
      maxY = Math.max(maxY, annotation.points[i + 1]);
    }
    return (
      maxX - minX < MIN_DRAW_SIZE && maxY - minY < MIN_DRAW_SIZE
    );
  }

  return false;
}

export function createAnnotation(
  partial: Partial<Annotation> & Pick<Annotation, "tool">,
  style: EditorStyle,
): Annotation {
  const fill =
    partial.tool === "highlight"
      ? partial.fill ?? style.stroke
      : partial.fill;

  return {
    id: partial.id ?? crypto.randomUUID(),
    tool: partial.tool,
    points: partial.points ?? [],
    x: partial.x ?? 0,
    y: partial.y ?? 0,
    width: partial.width ?? 0,
    height: partial.height ?? 0,
    text: partial.text ?? "",
    stroke: partial.stroke ?? style.stroke,
    strokeWidth: partial.strokeWidth ?? style.strokeWidth,
    fontSize: partial.fontSize ?? style.fontSize,
    fill,
    blurImageDataUrl: partial.blurImageDataUrl,
  };
}

export function cloneAnnotations(annotations: Annotation[]): Annotation[] {
  return JSON.parse(JSON.stringify(annotations)) as Annotation[];
}

export function measureTextBlock(
  text: string,
  fontSize: number,
  fontFamily = "system-ui, sans-serif",
): { width: number; height: number } {
  const lines = text.split("\n");
  const canvas = document.createElement("canvas");
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    return { width: 120, height: fontSize * 1.25 };
  }

  ctx.font = `${fontSize}px ${fontFamily}`;
  let maxWidth = 0;
  for (const line of lines) {
    maxWidth = Math.max(maxWidth, ctx.measureText(line || " ").width);
  }

  const lineHeight = fontSize * 1.25;
  return {
    width: Math.max(120, maxWidth + 16),
    height: Math.max(lineHeight, lines.length * lineHeight),
  };
}

export function getHighlightFill(color: string): string {
  return color;
}

export function getHighlightOpacity(): number {
  return HIGHLIGHT_OPACITY;
}

export function offsetPoints(
  points: number[],
  dx: number,
  dy: number,
): number[] {
  return points.map((value, index) =>
    index % 2 === 0 ? value + dx : value + dy,
  );
}

export function scaleArrowPoints(
  points: number[],
  scaleX: number,
  scaleY: number,
): number[] {
  const cx = (points[0] + points[2]) / 2;
  const cy = (points[1] + points[3]) / 2;

  return [
    cx + (points[0] - cx) * scaleX,
    cy + (points[1] - cy) * scaleY,
    cx + (points[2] - cx) * scaleX,
    cy + (points[3] - cy) * scaleY,
  ];
}

export function applyAnnotationDragEnd(
  annotation: Annotation,
  nodeX: number,
  nodeY: number,
): { changed: boolean } {
  if (annotation.tool === "arrow" || annotation.tool === "pen") {
    if (nodeX === 0 && nodeY === 0) {
      return { changed: false };
    }
    annotation.points = offsetPoints(annotation.points, nodeX, nodeY);
    return { changed: true };
  }

  if (annotation.tool === "text") {
    if (nodeX === annotation.x && nodeY === annotation.y) {
      return { changed: false };
    }
    annotation.x = nodeX;
    annotation.y = nodeY;
    return { changed: true };
  }

  if (
    annotation.tool === "rect" ||
    annotation.tool === "highlight" ||
    annotation.tool === "blur"
  ) {
    const rect = normalizeRect(
      nodeX,
      nodeY,
      annotation.width,
      annotation.height,
    );
    if (
      rect.x === annotation.x &&
      rect.y === annotation.y &&
      rect.width === annotation.width &&
      rect.height === annotation.height
    ) {
      return { changed: false };
    }
    annotation.x = rect.x;
    annotation.y = rect.y;
    annotation.width = rect.width;
    annotation.height = rect.height;
    return { changed: true };
  }

  return { changed: false };
}
