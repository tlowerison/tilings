import React, { MutableRefObject, ReactElement, forwardRef, useCallback, useEffect } from "react";

const pointHandlers = [
  "onMouseDown",
  "onMouseDownCapture",
  "onMouseEnter",
  "onMouseEnterCapture",
  "onMouseLeave",
  "onMouseLeaveCapture",
  "onMouseMove",
  "onMouseMoveCapture",
  "onMouseOut",
  "onMouseOutCapture",
  "onMouseOver",
  "onMouseOverCapture",
  "onMouseUp",
  "onMouseUpCapture",
  "onTouchCancel",
  "onTouchCancelCapture",
  "onTouchEnd",
  "onTouchEndCapture",
  "onTouchMove",
  "onTouchMoveCapture",
  "onTouchStart",
  "onTouchStartCapture",
] as const;

type PointHandler = (canvas: HTMLCanvasElement, x: number, y: number) => void;

export type Props = {
  height: number;
  width: number;
} & {
  [K in ArrayValue<typeof pointHandlers>]?: PointHandler;
};

export const Canvas = forwardRef<HTMLCanvasElement, Props>(
  ({ height, width, ...handlers }, ref) => {
    const mutableRef = ref as MutableRefObject<HTMLCanvasElement> | null;

    useEffect(
      () => {
        const canvas = mutableRef?.current;
        if (canvas) {
          canvas.height = height;
          canvas.width = width;
          canvas.getContext("2d")?.translate(width / 2, height / 2);
        }
      },
      [mutableRef?.current],
    );

    const wrapHandler = useCallback(
      (handler: PointHandler) => (({ clientX, clientY }) => {
        const canvas = mutableRef?.current;
        if (canvas) {
          const rect = canvas.getBoundingClientRect();
          const x = (clientX - rect.left) / (rect.right - rect.left) * canvas.width - width / 2;
          const y = (clientY - rect.top) / (rect.bottom - rect.top) * canvas.height - height / 2;
          handler(canvas, x, y);
        }
      }),
      [mutableRef?.current],
    );

    return (
      <canvas
        ref={ref || undefined}
        {...Object.fromEntries(Object.entries(handlers).map(([key, handler]) => [key, wrapHandler(handler)]))}
      />
    ) as ReactElement;
  },
);

export const clearCanvas = (canvas: HTMLCanvasElement | null, preserveTransform: boolean) => {
  const canvasContext = canvas?.getContext("2d");
  if (canvasContext) {
    canvasContext.beginPath();
    if (preserveTransform) {
      canvasContext.save();
      canvasContext.canvas.style.opacity = "0%";
      canvasContext.setTransform(1, 0, 0, 1, 0, 0);
    }

    canvasContext.clearRect(0, 0, canvasContext.canvas.width, canvasContext.canvas.height);

    if (preserveTransform) {
      canvasContext.restore();
      canvasContext.canvas.style.opacity = "100%";
    }
  }
};
