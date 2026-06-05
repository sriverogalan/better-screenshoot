export interface CaptureSessionStatus {
  currentCaptureId: string | null;
  incomingCaptureId: string;
  imagePreviewReady: boolean;
  historyEntries: number;
}

export function isCaptureSessionReady(status: CaptureSessionStatus): boolean {
  return (
    status.currentCaptureId === status.incomingCaptureId &&
    status.imagePreviewReady &&
    status.historyEntries > 0
  );
}
