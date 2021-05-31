import React, { useEffect, useRef, useState } from "react";
import { get_tilings, handle_event, set_tiling } from "pkg";
import { clearCanvas } from "utils";
import styles from "./styles.module.scss";

const canvasWidth = window.outerWidth - 100;
const canvasHeight = window.outerHeight - 150;
const translateX = canvasWidth / 2;
const translateY = canvasHeight / 2;

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

  useEffect(
    () => {
      const canvas = canvasRef.current;
      if (canvas) {
        canvas.width = canvasWidth;
        canvas.height = canvasHeight;
        canvas.style.width = canvasWidth + "px";
        canvas.style.height = canvasHeight + "px";
        canvas.getContext("2d")?.translate(translateX, translateY);
      }
    },
    [canvasRef.current],
  );

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
      <canvas
        ref={canvasRef}
        onMouseMove={({ clientX, clientY }) => {
          const canvas = canvasRef.current;
          if (canvas) {
            const rect = canvas.getBoundingClientRect();
            const x = (clientX - rect.left) / (rect.right - rect.left) * canvas.width - translateX;
            const y = (clientY - rect.top) / (rect.bottom - rect.top) * canvas.height - translateY;
            handle_event(canvas, x, y);
          }
        }}
      />
    </>
  );
};
