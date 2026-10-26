import { describe, expect, it, vi } from "vitest";
import {
  PIXELATE_BLOCK_SIZE,
  createPixelatedRegionDataUrl,
} from "./blur";

function mockImage(width: number, height: number): HTMLImageElement {
  return { width, height, naturalWidth: width, naturalHeight: height } as HTMLImageElement;
}

describe("createPixelatedRegionDataUrl", () => {
  it("returns empty string for zero-size region", async () => {
    const result = await createPixelatedRegionDataUrl(mockImage(100, 100), 10, 10, 0, 0);
    expect(result).toBe("");
  });

  it("draws a mosaic via downscale then nearest-neighbor upscale", async () => {
    const drawCalls: Array<{ w: number; h: number; dw: number; dh: number }> = [];
    const contexts: Array<{ imageSmoothingEnabled: boolean }> = [];

    const originalCreateElement = document.createElement.bind(document);
    vi.spyOn(document, "createElement").mockImplementation((tag: string) => {
      if (tag !== "canvas") return originalCreateElement(tag);
      const canvas = {
        width: 0,
        height: 0,
        getContext: () => {
          const ctx = {
            imageSmoothingEnabled: true,
            drawImage: (...args: unknown[]) => {
              // drawImage(img, sx, sy, sw, sh, dx, dy, dw, dh)
              if (args.length >= 9) {
                drawCalls.push({
                  w: args[3] as number,
                  h: args[4] as number,
                  dw: args[7] as number,
                  dh: args[8] as number,
                });
              }
            },
          };
          contexts.push(ctx);
          return ctx;
        },
        toDataURL: () => "data:image/png;base64,PIXELATED",
      };
      return canvas as unknown as HTMLCanvasElement;
    });

    const result = await createPixelatedRegionDataUrl(
      mockImage(200, 200),
      0,
      0,
      100,
      50,
      PIXELATE_BLOCK_SIZE,
    );

    expect(result).toBe("data:image/png;base64,PIXELATED");
    expect(contexts.length).toBe(2);
    expect(contexts.every((c) => c.imageSmoothingEnabled === false)).toBe(true);
    expect(drawCalls.length).toBe(2);
    expect(drawCalls[0].dw).toBe(Math.ceil(100 / PIXELATE_BLOCK_SIZE));
    expect(drawCalls[0].dh).toBe(Math.ceil(50 / PIXELATE_BLOCK_SIZE));
    expect(drawCalls[1].dw).toBe(100);
    expect(drawCalls[1].dh).toBe(50);

    vi.restoreAllMocks();
  });
});
