import { click, init, set_tiling } from "cellular-automata";

const canvas = document.getElementById("canvas");

let point = [];
let edges = [];

global.canvas = canvas;
global.set_tiling = (tiling_type, custom_config_str) => set_tiling(canvas, tiling_type, JSON.stringify(custom_config_str));
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
}

let locked = false;

/** Setup canvas to properly handle high DPI and redraw current plot. */
function setupCanvas() {
	const dpr = window.devicePixelRatio || 1.0;
  const aspectRatio = canvas.width / canvas.height;
  // const size = canvas.parentNode.offsetWidth * 0.8;
  const size = canvas.width;
  canvas.style.width = size + "px";
  canvas.style.height = size / aspectRatio + "px";
  canvas.width = size;
  canvas.height = size / aspectRatio;
  const context = canvas.getContext("2d");
  const translateX = canvas.width / 2
  const translateY = canvas.height / 2;
  context.translate(translateX, translateY);

  canvas.addEventListener("mousemove", ({ clientX, clientY }) => {
    const rect = canvas.getBoundingClientRect();
    const x = (clientX - rect.left) / (rect.right - rect.left) * canvas.width - translateX;
    const y = (clientY - rect.top) / (rect.bottom - rect.top) * canvas.height - translateY;
    (async () => {
      if (!locked) {
        locked = true;
        console.log(click(canvas, x, y));
        locked = false;
      }
    })();
  }, false);
}
