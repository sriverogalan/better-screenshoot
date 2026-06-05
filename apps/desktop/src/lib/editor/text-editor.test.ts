import { describe, expect, it } from "vitest";
import { DEFAULT_EDITOR_STYLE } from "./types";
import {
  appendTextAnnotation,
  createTextAnnotationAt,
  createTextEditorState,
  updateTextEditorState,
} from "./text-editor";

describe("text-editor", () => {
  it("crea anotación de texto en la posición indicada", () => {
    const annotation = createTextAnnotationAt({ x: 40, y: 60 }, DEFAULT_EDITOR_STYLE);

    expect(annotation.tool).toBe("text");
    expect(annotation.x).toBe(40);
    expect(annotation.y).toBe(60);
    expect(annotation.text).toBe("");
    expect(annotation.fontSize).toBe(DEFAULT_EDITOR_STYLE.fontSize);
  });

  it("añade anotación de texto al array existente", () => {
    const result = appendTextAnnotation([], { x: 10, y: 20 }, DEFAULT_EDITOR_STYLE);

    expect(result.annotations).toHaveLength(1);
    expect(result.annotation.id).toBe(result.annotations[0].id);
    expect(result.annotations[0].tool).toBe("text");
  });

  it("actualiza el valor del editor de texto sin mutar el id", () => {
    const editor = createTextEditorState("text-1", "");
    const updated = updateTextEditorState(editor, "hola");

    expect(updated).toEqual({ annotationId: "text-1", value: "hola" });
    expect(editor.value).toBe("");
  });

  it("devuelve null al actualizar sin editor activo", () => {
    expect(updateTextEditorState(null, "hola")).toBeNull();
  });
});
