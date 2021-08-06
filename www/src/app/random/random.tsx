import React, { useEffect, useMemo, useRef } from "react";
import styles from "./styles.module.scss";
import { Canvas, clearCanvas, makeHandlerConverters } from "components";
import { insertTileByPoint, removeTiling, setTiling, tilingSearch } from "client";
import { isMobile, newGlobalId } from "utils";

type CanvasConfig = {
  globalId: number;
  containerRef: React.RefObject<HTMLDivElement>;
  subContainerRef: React.RefObject<HTMLDivElement>;
};

type Process = {
  numTilesAdded: number;
  proceed: boolean;
  timeout: NodeJS.Timeout;
};

const canvasWidth = window.outerWidth;
const canvasHeight = window.outerHeight;

const clamp = (val: number, lower: number, upper: number) => Math.max(Math.min(val, upper), lower);

const easeOutFactory = (rate: number) => {
	// There's a discontinuity at rate = 0 that
	// doesn't make sense from a usability perspective
	// so patch it over.
	rate = (rate === 0) ? 1e-7 : rate;

	const sigmoid = (t: number) => (1 / (1 + Math.exp(-rate * t))) - 0.5;

	return (t: number) => {
		t = clamp(t, 0, 1);
		return sigmoid(t) / sigmoid(1);
	};
};

const rate = 4;
const easeOut = easeOutFactory(rate);
const minAddTileTimer = isMobile ? 5 : 30;
const maxAddTileTimer = isMobile ? 100 : 100;
const numTilesAddedCriticalPoint = isMobile ? 500 : 100;
const nextTimer = (numTilesAdded: number) => clamp(maxAddTileTimer * easeOut(numTilesAdded / numTilesAddedCriticalPoint), minAddTileTimer, Infinity);

const fadeOutDelta = 0.007;
const fadeOutDelayMs = 4 * 1000;
const maxOpacity = 0.9;

const runTiling = async ({ containerRef, globalId, subContainerRef }: CanvasConfig): Promise<Process> => {
  let process = { numTilesAdded: 0, proceed: true, timeout: setTimeout(() => {}, 0) };
  const tilings = await tilingSearch(".");

  const run = () => {
    const container = containerRef?.current;
    if (container) {
      const tiling = tilings[Math.floor(Math.random() * tilings.length)];
      setTiling(globalId, tiling.id);
      clearCanvas(globalId);
      container.style.opacity = `${maxOpacity}`;

      // move new process to first position in array
      process = randomlyAddTiles({ containerRef, globalId, subContainerRef });

      // fade out canvas 0
      setTimeout(
        () => {
          let opacity = maxOpacity;
          const fadeOutLoop = () => {
            if (opacity <= 0) {
              clearTimeout(process.timeout);
              process.proceed = false;
              run();
            } else {
              if (opacity - fadeOutDelta <= 0) {
                opacity = 0;
              } else {
                opacity = opacity - fadeOutDelta;
              }
              container.style.opacity = `${opacity}`;
              requestAnimationFrame(fadeOutLoop);
            }
          };
          fadeOutLoop();
        },
        fadeOutDelayMs,
      );
    }
  };

  run();

  return process;
};

const generateRandomPoint = () => ({
  clientX: Math.random() * canvasWidth,
  clientY: Math.random() * canvasHeight,
});

const randomlyAddTiles = ({ containerRef, globalId, subContainerRef, }: CanvasConfig): Process => {
  const process = {
    numTilesAdded: 0,
    proceed: true,
    timeout: setTimeout(() => {}, 0),
  };

  const handler = makeHandlerConverters({
    containerRef,
    globalId,
    subContainerRef,
    height: canvasHeight,
    width: canvasWidth,
  })(insertTileByPoint).mouse;

  const processFn = () => {
    if (process.proceed) {
      handler(generateRandomPoint() as MouseEvent);
      process.numTilesAdded += 1;
      process.timeout = setTimeout(processFn, nextTimer(process.numTilesAdded));
    }
  };

  process.timeout = setTimeout(processFn, nextTimer(process.numTilesAdded));

  return process;
};

export const Random = () => {
  const containerRef = useRef<HTMLDivElement>(null);
  const subContainerRef = useRef<HTMLDivElement>(null);
  const globalId = useMemo(() => newGlobalId(), []);

  useEffect(
    () => {
      if (containerRef.current && subContainerRef.current) {
        let process: Process;
        (async () => process = await runTiling({ containerRef, globalId, subContainerRef }))();
        return () => {
          process && clearTimeout(process.timeout);
          removeTiling(globalId);
        };
      }
    },
    [containerRef, subContainerRef],
  );

  return (
    <Canvas
      containerClassName={styles.container}
      containerRef={containerRef}
      globalId={globalId}
      height={canvasHeight}
      onMouseMove={insertTileByPoint}
      onTouchMove={insertTileByPoint}
      subContainerRef={subContainerRef}
      width={canvasWidth}
    />
  );
};
