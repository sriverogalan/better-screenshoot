import type { Annotation, EditorStyle, TextEditorState } from "./types";
import { createAnnotation } from "./utils";
import type { PointerPosition } from "./utils";

export function createTextAnnotationAt(
  pos: PointerPosition,
  style: EditorStyle,
): Annotation {
  return createAnnotation({ tool: "text", x: pos.x, y: pos.y }, style);
}

export function appendTextAnnotation(
  annotations: Annotation[],
  pos: PointerPosition,
  style: EditorStyle,
): { annotations: Annotation[]; annotation: Annotation } {
  const annotation = createTextAnnotationAt(pos, style);
  return {
    annotations: [...annotations, annotation],
    annotation,
  };
}

export function updateTextEditorState(
  editor: TextEditorState | null,
  value: string,
): TextEditorState | null {
  if (!editor) return null;
  return { ...editor, value };
}

export function createTextEditorState(
  annotationId: string,
  text: string,
): TextEditorState {
  return { annotationId, value: text };
}
