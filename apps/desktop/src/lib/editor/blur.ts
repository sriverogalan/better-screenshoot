import { normalizeRect } from "./utils";

/** Edge length of each mosaic cell in source pixels. */
export const PIXELATE_BLOCK_SIZE = 10;

/**
 * Pixelates a rectangular region of `sourceImage` (mosaic) and returns a PNG data URL.
 * Coordinates are in the source image's native pixel space.
 */
export async function createPixelatedRegionDataUrl(
  sourceImage: HTMLImageElement,
  x: number,
  y: number,
  width: number,
  height: number,
  blockSize = PIXELATE_BLOCK_SIZE,
): Promise<string> {
  const rect = normalizeRect(x, y, width, height);
  const sx = Math.round(rect.x);
  const sy = Math.round(rect.y);
  const sw = Math.round(rect.width);
  const sh = Math.round(rect.height);

  if (sw < 1 || sh < 1) {
    return "";
  }

  const cell = Math.max(2, Math.round(blockSize));
  const smallW = Math.max(1, Math.ceil(sw / cell));
  const smallH = Math.max(1, Math.ceil(sh / cell));

  const small = document.createElement("canvas");
  small.width = smallW;
  small.height = smallH;
  const smallCtx = small.getContext("2d");
  if (!smallCtx) return "";

  smallCtx.imageSmoothingEnabled = false;
  smallCtx.drawImage(sourceImage, sx, sy, sw, sh, 0, 0, smallW, smallH);

  const canvas = document.createElement("canvas");
  canvas.width = sw;
  canvas.height = sh;
  const ctx = canvas.getContext("2d");
  if (!ctx) return "";

  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(small, 0, 0, smallW, smallH, 0, 0, sw, sh);

  return canvas.toDataURL("image/png");
}

/** @deprecated Use createPixelatedRegionDataUrl */
export const createBlurredRegionDataUrl = createPixelatedRegionDataUrl;
