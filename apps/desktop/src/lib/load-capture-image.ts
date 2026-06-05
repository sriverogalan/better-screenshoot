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
    throw new Error("La captura no tiene imagen");
  }

  return convertFileSrc(capture.file_path);
}

async function readCaptureDataUrlFromDisk(filePath: string): Promise<string> {
  return invoke<string>("read_capture_data_url", { filePath });
}

export function loadHtmlImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    // WKWebView/Safari marca el canvas como "tainted" sin esto y
    // canvas.toDataURL() lanza "The operation is insecure".
    img.crossOrigin = "anonymous";
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error("Error al decodificar la imagen"));
    img.src = src;
  });
}

export async function loadCaptureImage(
  capture: SavedCapture,
): Promise<LoadedCaptureImage> {
  if (!capture.data_url && !capture.file_path) {
    throw new Error("La captura no tiene imagen");
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
  // Sin blob URLs que revocar; se mantiene por compatibilidad con el editor.
}
