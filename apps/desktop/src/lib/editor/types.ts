export type Tool =
  | "select"
  | "arrow"
  | "rect"
  | "text"
  | "highlight"
  | "pen"
  | "blur";

export interface EditorStyle {
  stroke: string;
  strokeWidth: number;
  fontSize: number;
}

export interface Annotation {
  id: string;
  tool: Tool;
  points: number[];
  x: number;
  y: number;
  width: number;
  height: number;
  text: string;
  stroke: string;
  strokeWidth: number;
  fontSize: number;
  fill?: string;
  blurImageDataUrl?: string;
}

export interface DisplayLayout {
  stageWidth: number;
  stageHeight: number;
  scale: number;
  offsetX: number;
  offsetY: number;
  displayW: number;
  displayH: number;
  zoomPercent: number;
  imageWidth: number;
  imageHeight: number;
}

export interface TextEditorState {
  annotationId: string;
  value: string;
}

export const DEFAULT_EDITOR_STYLE: EditorStyle = {
  stroke: "#5b8def",
  strokeWidth: 3,
  fontSize: 18,
};

export const HEADER_HEIGHT = 96;
export const MIN_DRAW_SIZE = 4;
export const HIGHLIGHT_OPACITY = 0.35;

export const COLOR_PRESETS = [
  "#5b8def",
  "#ef4444",
  "#22c55e",
  "#facc15",
  "#ffffff",
  "#000000",
  "#f97316",
  "#a855f7",
] as const;

export const STROKE_WIDTHS = [2, 4, 6] as const;
export const FONT_SIZES = [14, 18, 24, 32] as const;

export type DraftAnnotation = Omit<Annotation, "id" | "text"> & {
  text?: string;
};
