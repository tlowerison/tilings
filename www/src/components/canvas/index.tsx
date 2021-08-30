import React, { RefObject, useCallback, useEffect, useMemo, useRef, useState } from "react";
import styles from "./styles.module.scss";
// @ts-ignore
import { getContainer, getSubContainer, getSubContainerDetail, onScroll, setContainers } from "./utils";

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

export const clearCanvas = (globalId: number) => {
  // @ts-ignore: https://developer.mozilla.org/en-US/docs/Web/API/Element/replaceChildren
  getSubContainerDetail(globalId)?.element?.replaceChildren();
  setContainers(globalId, getContainer(globalId), getSubContainer(globalId));
};

type RustPointHandler = (globalId: number, x: number, y: number) => void;
type JsPointMouseHandler = (event: MouseEvent) => void;
type JsPointTouchHandler = (event: TouchEvent) => void;

export type CanvasEventHandlerHelper = {
  globalId: number;
  height: number;
  width: number;
  containerRef?: React.RefObject<HTMLElement> | null;
  numRetriesOnFail?: number;
  subContainerRef?: React.RefObject<HTMLElement> | null;
  zoomRef?: { current: number };
};

export const canvasXOffset = 12;
export const canvasYOffset = 763;

export const makeHandlerConverters = ({ containerRef, globalId, numRetriesOnFail = 0, subContainerRef }: CanvasEventHandlerHelper) =>
  (handler: RustPointHandler): {
    mouse: JsPointMouseHandler;
    touch: JsPointTouchHandler;
  } => {
    const container = containerRef?.current;
    const subContainer = subContainerRef?.current;

    if (!container || !subContainer) {
      return { mouse: () => {}, touch: () => {} };
    }

    const containerRect = container.getBoundingClientRect();
    const subContainerRect = subContainer.getBoundingClientRect();

    const xScale = subContainerRect.width / (subContainerRect.right - subContainerRect.left);
    const yScale = subContainerRect.height / (subContainerRect.bottom - subContainerRect.top);

    const jsPointHandler = (clientX: number, clientY: number) => {
      const subContainerDetail = getSubContainerDetail(globalId);
      if (subContainerDetail) {
        const containerX = clientX - containerRect.left;
        const containerY = clientY - containerRect.top;
        const x = xScale * (containerX + container.scrollLeft - subContainerDetail.left - containerRect.width / 2 - canvasXOffset);
        const y = yScale * (containerY + container.scrollTop - subContainerDetail.top - canvasYOffset);
        for (let i = 0; i < numRetriesOnFail + 1; i += 1) {
          try {
            handler(globalId, x, y);
            break;
          } catch (_) {}
        }
      }
    };

    return {
      mouse: ({ clientX, clientY }) => jsPointHandler(clientX, clientY),
      touch: ({ touches = [] }) => {
        for (const touch of touches as Touch[]) {
          jsPointHandler(touch.clientX, touch.clientY);
        }
      }
    };
  };

export type Props = {
  containerClassName?: string;
  containerRef?: RefObject<HTMLDivElement> | undefined;
  globalId: number;
  height: number;
  scrollable?: boolean;
  subContainerClassName?: string;
  subContainerRef?: RefObject<HTMLDivElement> | undefined;
  width: number;
} & {
  [K in ArrayValue<typeof mouseHandlers>]?: RustPointHandler;
} & {
  [K in ArrayValue<typeof touchHandlers>]?: RustPointHandler;
};

export const Canvas = (
  {
    containerClassName,
    containerRef: containerRefNullable,
    globalId,
    height,
    scrollable,
    subContainerClassName,
    subContainerRef: subContainerRefNullable,
    width,
    ...handlers
  }: Props,
) => {
    let containerRef = useRef<HTMLDivElement>(null);
    containerRef = containerRefNullable || containerRef;

    let subContainerRef = useRef<HTMLDivElement>(null);
    subContainerRef = subContainerRefNullable || subContainerRef;

    const zoomRef = useRef(1);

    const [hasAddedListeners, setHasAddedListeners] = useState(false);

    const handlerConverters = useCallback(
      makeHandlerConverters({ globalId, height, width, containerRef, subContainerRef, zoomRef }),
      [globalId, containerRef, height, subContainerRef, width, zoomRef],
    );

    const convertedHandlers = useMemo(
      () => Object.fromEntries(Object.entries(handlers).map(([key, handler]) => {
        const handlers = handlerConverters(handler);
        return [
          key,
          key in mouseHandlerSet ? handlers.mouse : handlers.touch,
        ];
      })),
      [handlers, handlerConverters],
    );

    const style = useMemo(() => ({ height: `${height}px`, width: `${width}px` }), [height, width]);

    useEffect(
      () => {
        const container = containerRef?.current;
        const subContainer = subContainerRef?.current;

        if (container && !hasAddedListeners) {
            setHasAddedListeners(true);
        }
        if (container && subContainer) {
          setContainers(globalId, container, subContainer);
          onScroll(globalId);
        }
        return () => {
          setContainers(null, null);
        };
      },
      [containerRef, globalId, subContainerRef],
    );

    return (
      <div
        {...convertedHandlers}
        className={`${styles.container} ${containerClassName || ""} ${scrollable && styles.scrollable}`}
        onScroll={() => onScroll(globalId)}
        ref={containerRef}
        style={style}
      >
        <div
          className={`${styles.subContainer} ${subContainerClassName || ""}`}
          ref={subContainerRef}
          style={style}
        />
      </div>
    );
};
