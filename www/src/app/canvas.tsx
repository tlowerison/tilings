import React, { MutableRefObject, ReactElement, forwardRef, useCallback, useEffect, useMemo } from "react";

const mouseHandlers = [
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
] as const;

const touchHandlers = [
  "onTouchCancel",
  "onTouchCancelCapture",
  "onTouchEnd",
  "onTouchEndCapture",
  "onTouchMove",
  "onTouchMoveCapture",
  "onTouchStart",
  "onTouchStartCapture",
] as const;

const mouseHandlerSet = Object.fromEntries(mouseHandlers.map(name => [name, name]));

type PointHandler = (canvas: HTMLCanvasElement, x: number, y: number) => void;

export type Props = {
  height: number;
  width: number;
} & {
  [K in ArrayValue<typeof mouseHandlers>]?: PointHandler;
} & {
  [K in ArrayValue<typeof touchHandlers>]?: PointHandler;
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

    const handleEvent = useCallback(
      (canvas: HTMLCanvasElement, clientX: number, clientY: number, handler: PointHandler) => {
        const rect = canvas.getBoundingClientRect();
        const x = (clientX - rect.left) / (rect.right - rect.left) * canvas.width - width / 2;
        const y = (clientY - rect.top) / (rect.bottom - rect.top) * canvas.height - height / 2;
        handler(canvas, x, y);
      },
      [height, width],
    );

    const wrapMouseHandler = useMemo(
      () => {
        const canvas = mutableRef?.current;
        if (canvas) {
          return (handler: PointHandler) => ({ clientX, clientY }) => handleEvent(canvas, clientX, clientY, handler);
        } else {
          return (_: PointHandler) => {};
        }
      },
      [handleEvent, mutableRef?.current],
    );

    const wrapTouchHandler = useMemo(
      () => {
        const canvas = mutableRef?.current;
        if (canvas) {
          return (handler: PointHandler) => ({ touches = [] }: { touches: Touch[] }) => {
            for (const touch of touches) {
              handleEvent(canvas, touch.clientX, touch.clientY, handler);
            }
          };
        } else {
          return (_: PointHandler) => {};
        }
      },
      [handleEvent, mutableRef?.current],
    );

    return (
      <canvas
        ref={ref || undefined}
        {...Object.fromEntries(Object.entries(handlers).map(([key, handler]) => [
          key,
          key in mouseHandlerSet ? wrapMouseHandler(handler) : wrapTouchHandler(handler),
        ]))}
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
