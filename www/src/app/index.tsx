import React, { useEffect, useRef, useState } from "react";
import { Canvas, clearCanvas } from "./canvas";
import { get_tilings, handle_event, set_tiling } from "pkg";
import styles from "./styles.module.scss";

const isMobile = () => window.outerWidth > 800;

const canvasWidth = isMobile()
  ? window.outerWidth - 100
  : window.outerWidth;
const canvasHeight = isMobile()
  ? window.outerHeight - 150
  : window.outerHeight - 120;

export const App = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const selectRef = useRef<HTMLSelectElement>(null);

  const [tilings, setTilings] = useState({});
  const [selected, setSelected] = useState("");

  useEffect(() => {
    const tilings = get_tilings();
    if (tilings && typeof tilings === "string") {
      setTilings(JSON.parse(tilings));
    }
  }, []);

  return (
    <>
      <h1 className={styles.title}>Tilings</h1>
      <select
        ref={selectRef}
        name="Tiling"
        className={styles.select}
        onChange={event => {
          // @ts-ignore
          const selected = event?.target?.value || "";
          const canvas = canvasRef.current;
          setSelected(selected);

          if (canvas) {
            clearCanvas(canvas, true);
            set_tiling(canvas, selected);
          }
        }}
      >
        <option value="">-</option>
        {Object.keys(tilings).map(name => (
          <option value={name} selected={name === selected}>{name}</option>
        ))}
      </select>
      <Canvas
        ref={canvasRef}
        height={canvasHeight}
        width={canvasWidth}
        onMouseMove={handle_event}
        onTouchMove={handle_event}
      />
    </>
  );
};
