import { describe, it, expect, beforeEach } from "vitest"
import { useEditorHistory } from "./useEditorHistory"

describe("useEditorHistory", () => {
  it("starts with empty annotations, empty history, and index -1", () => {
    const { annotations, history, historyIndex } = useEditorHistory()
    expect(annotations.value).toEqual([])
    expect(history.value).toEqual([])
    expect(historyIndex.value).toBe(-1)
  })

  it("pushHistory adds a snapshot and increments index", () => {
    const { annotations, history, historyIndex, pushHistory } = useEditorHistory()

    annotations.value = [{ id: "a1", tool: "arrow" } as any]
    pushHistory()

    expect(history.value).toHaveLength(1)
    expect(historyIndex.value).toBe(0)
  })

  it("undo returns true and restores the previous snapshot", () => {
    const { annotations, history, historyIndex, pushHistory, undo } = useEditorHistory()

    annotations.value = [{ id: "a1", tool: "arrow" } as any]
    pushHistory()
    annotations.value = [{ id: "a2", tool: "rect" } as any]
    pushHistory()

    const result = undo()

    expect(result).toBe(true)
    expect(historyIndex.value).toBe(0)
    expect(annotations.value).toHaveLength(1)
    expect(annotations.value[0].id).toBe("a1")
  })

  it("undo returns false when already at the first entry", () => {
    const { annotations, pushHistory, undo } = useEditorHistory()

    annotations.value = [{ id: "a1", tool: "arrow" } as any]
    pushHistory()

    const result = undo()
    expect(result).toBe(false)
  })

  it("redo returns true and moves index forward", () => {
    const { annotations, pushHistory, undo, redo } = useEditorHistory()

    annotations.value = [{ id: "a1", tool: "arrow" } as any]
    pushHistory()
    annotations.value = [{ id: "a2", tool: "rect" } as any]
    pushHistory()
    undo()

    const result = redo()
    expect(result).toBe(true)
    expect(annotations.value[0].id).toBe("a2")
  })

  it("redo returns false when already at the latest entry", () => {
    const { annotations, pushHistory, redo } = useEditorHistory()

    annotations.value = [{ id: "a1", tool: "arrow" } as any]
    pushHistory()

    expect(redo()).toBe(false)
  })

  it("resetHistory clears annotations, history, and index", () => {
    const { annotations, history, historyIndex, pushHistory, resetHistory } = useEditorHistory()

    annotations.value = [{ id: "a1", tool: "arrow" } as any]
    pushHistory()
    resetHistory()

    expect(annotations.value).toEqual([])
    expect(history.value).toEqual([])
    expect(historyIndex.value).toBe(-1)
  })

  it("initHistory creates the baseline history entry", () => {
    const { annotations, history, historyIndex, initHistory } = useEditorHistory()

    annotations.value = [{ id: "a1", tool: "arrow" } as any]
    initHistory()

    expect(history.value).toHaveLength(1)
    expect(historyIndex.value).toBe(0)
  })

  it("pushHistory after undo truncates redo future", () => {
    const { annotations, history, pushHistory, undo } = useEditorHistory()

    annotations.value = [{ id: "a1", tool: "arrow" } as any]
    pushHistory()
    annotations.value = [{ id: "a2", tool: "rect" } as any]
    pushHistory()
    undo()

    annotations.value = [{ id: "a3", tool: "pen" } as any]
    pushHistory()

    expect(history.value).toHaveLength(2)
    expect(history.value[1][0].id).toBe("a3")
  })
})
