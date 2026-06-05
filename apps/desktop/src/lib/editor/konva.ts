export interface KonvaStage {
  findOne: (selector: string) => KonvaNode | null;
  getPointerPosition: () => { x: number; y: number } | null;
  toDataURL: (config?: {
    pixelRatio?: number;
    x?: number;
    y?: number;
    width?: number;
    height?: number;
  }) => string;
}

export interface KonvaNode {
  getClassName: () => string;
  name: () => string;
  getStage: () => KonvaStage;
  x: (value?: number) => number;
  y: (value?: number) => number;
  scaleX: (value?: number) => number;
  scaleY: (value?: number) => number;
  width: (value?: number) => number;
  height: (value?: number) => number;
  points: (value?: number[]) => number[];
}

export interface KonvaTransformerNode {
  nodes: (nodes: KonvaNode[]) => void;
  resizeEnabled: (value: boolean) => void;
  rotateEnabled: (value: boolean) => void;
  enabledAnchors: (anchors: string[]) => void;
  getLayer: () => { batchDraw: () => void } | null;
}

export interface KonvaTransformerRef {
  getNode: () => KonvaTransformerNode;
}

export interface KonvaStageRef {
  getStage: () => KonvaStage;
}
