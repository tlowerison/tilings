import { handle_event, init, set_tiling } from "cellular-automata";

const canvas = document.getElementById("canvas");
const canvasContext = canvas.getContext("2d");
const select = document.getElementById("select-tiling");
const setTiling = tilingType => canvasContext.clear(true) && console.log(set_tiling(canvas, tilingType));
let translateX = 0;
let translateY = 0;

CanvasRenderingContext2D.prototype.clear = CanvasRenderingContext2D.prototype.clear || function (preserveTransform) {
  this.beginPath();
  if (preserveTransform) {
    this.save();
    this.canvas.style.opacity = "0%";
    this.setTransform(1, 0, 0, 1, 0, 0);
  }

  this.clearRect(0, 0, this.canvas.width, this.canvas.height);

  if (preserveTransform) {
    this.restore();
    this.canvas.style.opacity = "100%";
  }
  return true;
};

export const main = () => setupUI() && setupCanvas();

const setupUI = () => {
  const result = init();
  if (typeof result === "string") {
    const tiling_names = JSON.parse(result);
    for (let i = 0; i < tiling_names.length; i += 1) {
      const tiling_name = tiling_names[i];
      const option = document.createElement("option");
      option.setAttribute("value", tiling_name);
      option.innerHTML = tiling_name;
      select.appendChild(option);
    }
  }

  select.addEventListener("change", event => setTiling(event.target.value));

  canvas.addEventListener("mousemove", ({ clientX, clientY }) => {
    const rect = canvas.getBoundingClientRect();
    const x = (clientX - rect.left) / (rect.right - rect.left) * canvas.width - translateX;
    const y = (clientY - rect.top) / (rect.bottom - rect.top) * canvas.height - translateY;
    handle_event(canvas, x, y);
  }, false);
  return true;
};

const setupCanvas = () => {
  canvas.width = window.outerWidth - 100;
  canvas.height = window.outerHeight - 150;
	const dpr = window.devicePixelRatio || 1.0;
  const aspectRatio = canvas.width / canvas.height;
  const size = canvas.width;
  canvas.style.width = size + "px";
  canvas.style.height = size / aspectRatio + "px";
  translateX = canvas.width / 2;
  translateY = canvas.height / 2;
  canvasContext.translate(translateX, translateY);
  return true;
};
