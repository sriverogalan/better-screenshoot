import { describe, expect, it } from "vitest";
import type { Annotation, TextEditorState } from "./editor/types";
import { commitPendingText } from "./editor-export";

const baseLines: Annotation[] = [
  {
    id: "text-1",
    tool: "text",
    points: [],
    x: 10,
    y: 20,
    width: 0,
    height: 0,
    text: "hola",
    stroke: "#5b8def",
    strokeWidth: 3,
    fontSize: 18,
  },
  {
    id: "arrow-1",
    tool: "arrow",
    points: [0, 0, 100, 100],
    x: 0,
    y: 0,
    width: 0,
    height: 0,
    text: "",
    stroke: "#5b8def",
    strokeWidth: 3,
    fontSize: 18,
  },
];

describe("commitPendingText", () => {
  it("aplica texto pendiente y marca historial como cambiado", () => {
    const editor: TextEditorState = { annotationId: "text-1", value: "  nuevo texto  " };

    const result = commitPendingText(editor, baseLines);

    expect(result.historyChanged).toBe(true);
    expect(result.lines.find((line) => line.id === "text-1")?.text).toBe("nuevo texto");
  });

  it("elimina anotación de texto vacía sin cambiar historial", () => {
    const editor: TextEditorState = { annotationId: "text-1", value: "   " };

    const result = commitPendingText(editor, baseLines);

    expect(result.historyChanged).toBe(false);
    expect(result.lines.some((line) => line.id === "text-1")).toBe(false);
    expect(result.lines).toHaveLength(1);
  });

  it("no modifica líneas si la anotación no existe", () => {
    const editor: TextEditorState = { annotationId: "missing", value: "texto" };

    const result = commitPendingText(editor, baseLines);

    expect(result.historyChanged).toBe(false);
    expect(result.lines).toEqual(baseLines);
  });
});
