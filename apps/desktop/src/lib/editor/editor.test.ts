import { describe, expect, it } from "vitest";
import { DEFAULT_EDITOR_STYLE } from "./types";
import {
  applyAnnotationDragEnd,
  cloneAnnotations,
  computeDisplayLayout,
  createAnnotation,
  getImagePointer,
  isAnnotationTooSmall,
  normalizeRect,
  offsetPoints,
  scaleArrowPoints,
} from "./utils";

describe("editor utils", () => {
  it("computes display layout with centered offsets", () => {
    const layout = computeDisplayLayout(800, 600, 400, 300);
    expect(layout.scale).toBe(0.5);
    expect(layout.displayW).toBe(400);
    expect(layout.displayH).toBe(300);
    expect(layout.offsetX).toBe(0);
    expect(layout.offsetY).toBe(0);
    expect(layout.zoomPercent).toBe(50);
  });

  it("converts stage pointer to image coordinates", () => {
    const layout = computeDisplayLayout(100, 100, 200, 200);
    const pointer = getImagePointer(60, 70, layout);
    expect(pointer).toEqual({ x: 30, y: 35 });
  });

  it("returns null for pointer outside image bounds", () => {
    const layout = computeDisplayLayout(100, 100, 200, 200);
    expect(getImagePointer(-5, 50, layout)).toBeNull();
    expect(getImagePointer(250, 50, layout)).toBeNull();
  });

  it("normalizes rectangles with negative width and height", () => {
    expect(normalizeRect(50, 50, -20, -30)).toEqual({
      x: 30,
      y: 20,
      width: 20,
      height: 30,
    });
  });

  it("detects annotations that are too small", () => {
    expect(
      isAnnotationTooSmall({
        tool: "arrow",
        points: [0, 0, 1, 1],
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        stroke: "#5b8def",
        strokeWidth: 3,
        fontSize: 18,
      }),
    ).toBe(true);

    expect(
      isAnnotationTooSmall({
        tool: "rect",
        points: [],
        x: 0,
        y: 0,
        width: 40,
        height: 20,
        stroke: "#5b8def",
        strokeWidth: 3,
        fontSize: 18,
      }),
    ).toBe(false);
  });

  it("creates annotations with style defaults", () => {
    const annotation = createAnnotation({ tool: "arrow", points: [0, 0, 10, 10] }, DEFAULT_EDITOR_STYLE);
    expect(annotation.stroke).toBe("#5b8def");
    expect(annotation.strokeWidth).toBe(3);
    expect(annotation.fontSize).toBe(18);
    expect(annotation.id).toBeTruthy();
  });

  it("clones annotations without shared references", () => {
    const source = [
      createAnnotation({ tool: "rect", x: 1, y: 2, width: 3, height: 4 }, DEFAULT_EDITOR_STYLE),
    ];
    const cloned = cloneAnnotations(source);
    cloned[0].x = 99;
    expect(source[0].x).toBe(1);
  });

  it("offsets and scales arrow points", () => {
    expect(offsetPoints([0, 0, 10, 10], 5, 5)).toEqual([5, 5, 15, 15]);
    expect(scaleArrowPoints([0, 0, 10, 0], 2, 1)).toEqual([-5, 0, 15, 0]);
  });

  it("persists absolute position when dragging rects", () => {
    const annotation = createAnnotation(
      { tool: "rect", x: 100, y: 80, width: 40, height: 20 },
      DEFAULT_EDITOR_STYLE,
    );

    const result = applyAnnotationDragEnd(annotation, 150, 120);

    expect(result).toEqual({ changed: true });
    expect(annotation.x).toBe(150);
    expect(annotation.y).toBe(120);
    expect(annotation.width).toBe(40);
    expect(annotation.height).toBe(20);
  });

  it("persists absolute position when dragging text", () => {
    const annotation = createAnnotation(
      { tool: "text", x: 100, y: 80, text: "Hola" },
      DEFAULT_EDITOR_STYLE,
    );

    const result = applyAnnotationDragEnd(annotation, 150, 120);

    expect(result).toEqual({ changed: true });
    expect(annotation.x).toBe(150);
    expect(annotation.y).toBe(120);
  });

  it("offsets arrow points when dragging line nodes", () => {
    const annotation = createAnnotation(
      { tool: "arrow", points: [0, 0, 10, 10] },
      DEFAULT_EDITOR_STYLE,
    );

    const result = applyAnnotationDragEnd(annotation, 5, 5);

    expect(result).toEqual({ changed: true });
    expect(annotation.points).toEqual([5, 5, 15, 15]);
  });

  it("returns unchanged when drag node position matches annotation", () => {
    const annotation = createAnnotation(
      { tool: "text", x: 100, y: 80, text: "Hola" },
      DEFAULT_EDITOR_STYLE,
    );

    const result = applyAnnotationDragEnd(annotation, 100, 80);

    expect(result).toEqual({ changed: false });
    expect(annotation.x).toBe(100);
    expect(annotation.y).toBe(80);
  });
});
