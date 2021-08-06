import React, { useEffect, useMemo, useRef } from "react";
import styles from "./styles.module.scss";
import { Canvas, canvasXOffset, canvasYOffset } from "components";
import { newGlobalId } from "utils";
import { insertTileByPoint, removeTiling, setTiling } from "client";

type Props = {
  height: number;
  tilingId: number;
  width: number;
};

const tileScale = 12;

export const Thumbnail = ({ height, tilingId, width }: Props) => {
  const globalId = useMemo(newGlobalId, []);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(
    () => {
      setTiling(globalId, tilingId);
      setTimeout(
        () => {
          for (let x = -width / 2 - canvasXOffset; x < width / 2 - canvasXOffset + 2 * tileScale; x += tileScale) {
            for (let y = -canvasYOffset; y < height - canvasYOffset + 2 * tileScale; y += tileScale) {
              try {
                insertTileByPoint(globalId, x, y);
              } catch (e) {}
            }
          }
        },
        500,
      );
    },
    [globalId, tilingId],
  );
  useEffect(() => () => removeTiling(globalId), [globalId]);

  return (
    <div
      className={styles.container}
      style={{ height: `${height}px`, width: `${width}px` }}
    >
      <Canvas
        containerRef={containerRef}
        globalId={globalId}
        height={height}
        width={width}
      />
    </div>
  );
};
