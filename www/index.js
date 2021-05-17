import { click, init, set_tiling } from "cellular-automata";

const canvas = document.getElementById("canvas");

let point = [];
let edges = [];

global.canvas = canvas;
global.set_tiling = (tiling_type) => set_tiling(canvas, tiling_type);
global.click = click;

/** Main entry point */
export function main() {
  setupUI();
  setupCanvas();
  console.log(init(canvas));
}

/** Add event listeners. */
function setupUI() {
  window.addEventListener("resize", setupCanvas);
  document.getElementById("step").onclick = () => global.step(canvas, Math.floor(Math.random() * edges.length));
}

/** Setup canvas to properly handle high DPI and redraw current plot. */
function setupCanvas() {
	const dpr = window.devicePixelRatio || 1.0;
  const aspectRatio = canvas.width / canvas.height;
  const size = canvas.parentNode.offsetWidth * 0.8;
  canvas.style.width = size + "px";
  canvas.style.height = size / aspectRatio + "px";
  canvas.width = size;
  canvas.height = size / aspectRatio;
  canvas.addEventListener("click", ({ clientX, clientY }) => {
    const rect = canvas.getBoundingClientRect();
    const x = (clientX - rect.left) / (rect.right - rect.left) * canvas.width;
    const y = (clientY - rect.top) / (rect.bottom - rect.top) * canvas.height;
    console.log(click(canvas, x, y));
  }, false);
}
