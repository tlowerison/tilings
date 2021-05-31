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
