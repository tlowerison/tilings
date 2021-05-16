import { init, set_tiling, step } from "cellular-automata";

const canvas = document.getElementById("canvas");

let point = [];
let edges = [];

global.canvas = canvas;
global.set_tiling = (tiling_type) => set_tiling(canvas, tiling_type);
global.step = (canvas, edge_index) => {
  const result = step(canvas, edge_index);
  if (typeof result === "string") {
    const vals = JSON.parse(result);
    point = vals.vertex_star_point;
    edges = vals.edges.map(edge => edge.map(point => point.map(num => Math.round(num * 1000) / 1000)));
  }
  return result;
};

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
}
