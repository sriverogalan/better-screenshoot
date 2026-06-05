import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { SavedCapture } from "./tauri";

export interface LoadedCaptureImage {
  element: HTMLImageElement;
  dataUrl: string;
}

export async function resolveCaptureDataUrl(
  capture: SavedCapture,
): Promise<string> {
  if (capture.data_url) {
    return capture.data_url;
  }

  if (!capture.file_path) {
    throw new Error("Capture has no image");
  }

  return convertFileSrc(capture.file_path);
}

async function readCaptureDataUrlFromDisk(filePath: string): Promise<string> {
  return invoke<string>("read_capture_data_url", { filePath });
}

export function loadHtmlImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    // WKWebView/Safari marks the canvas as "tainted" without this and
    // canvas.toDataURL() throws "The operation is insecure".
    img.crossOrigin = "anonymous";
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error("Error decoding image"));
    img.src = src;
  });
}

export async function loadCaptureImage(
  capture: SavedCapture,
): Promise<LoadedCaptureImage> {
  if (!capture.data_url && !capture.file_path) {
    throw new Error("Capture has no image");
  }

  const primarySrc = await resolveCaptureDataUrl(capture);

  try {
    const element = await loadHtmlImage(primarySrc);
    return { element, dataUrl: primarySrc };
  } catch (primaryError) {
    if (!capture.file_path || capture.data_url) {
      throw primaryError;
    }

    const fallbackSrc = await readCaptureDataUrlFromDisk(capture.file_path);
    const element = await loadHtmlImage(fallbackSrc);
    return { element, dataUrl: fallbackSrc };
  }
}

export function disposeCaptureImage() {
  // No blob URLs to revoke; kept for editor compatibility.
}
