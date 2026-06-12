import { describe, expect, it } from "vitest";
import { DEFAULT_EDITOR_STYLE } from "./types";
import {
  appendTextAnnotation,
  createTextAnnotationAt,
  createTextEditorState,
  updateTextEditorState,
} from "./text-editor";

describe("text-editor", () => {
  it("creates text annotation at the given position", () => {
    const annotation = createTextAnnotationAt({ x: 40, y: 60 }, DEFAULT_EDITOR_STYLE);

    expect(annotation.tool).toBe("text");
    expect(annotation.x).toBe(40);
    expect(annotation.y).toBe(60);
    expect(annotation.text).toBe("");
    expect(annotation.fontSize).toBe(DEFAULT_EDITOR_STYLE.fontSize);
  });

  it("appends text annotation to existing array", () => {
    const result = appendTextAnnotation([], { x: 10, y: 20 }, DEFAULT_EDITOR_STYLE);

    expect(result.annotations).toHaveLength(1);
    expect(result.annotation.id).toBe(result.annotations[0].id);
    expect(result.annotations[0].tool).toBe("text");
  });

  it("updates text editor value without mutating id", () => {
    const editor = createTextEditorState("text-1", "");
    const updated = updateTextEditorState(editor, "hola");

    expect(updated).toEqual({ annotationId: "text-1", value: "hola" });
    expect(editor.value).toBe("");
  });

  it("returns null when updating with no active editor", () => {
    expect(updateTextEditorState(null, "hola")).toBeNull();
  });
});
