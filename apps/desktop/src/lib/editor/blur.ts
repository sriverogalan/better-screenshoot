import { normalizeRect } from "./utils";

export async function createBlurredRegionDataUrl(
  sourceImage: HTMLImageElement,
  x: number,
  y: number,
  width: number,
  height: number,
  blurRadius = 8,
): Promise<string> {
  const rect = normalizeRect(x, y, width, height);
  const sx = Math.round(rect.x);
  const sy = Math.round(rect.y);
  const sw = Math.round(rect.width);
  const sh = Math.round(rect.height);

  if (sw < 1 || sh < 1) {
    return "";
  }

  const canvas = document.createElement("canvas");
  canvas.width = sw;
  canvas.height = sh;
  const ctx = canvas.getContext("2d");
  if (!ctx) return "";

  ctx.filter = `blur(${blurRadius}px)`;
  ctx.drawImage(sourceImage, sx, sy, sw, sh, 0, 0, sw, sh);

  return canvas.toDataURL("image/png");
}
