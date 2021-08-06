const canvasRadiusRefs = {};
const containerRefs = {};
const subContainerDetailRefs = {};

const defaultSubContainerDetail = {
  element: null,
  left: 0,
  top: 0,
  right: 0,
  bottom: 0,
  width: 0,
  height: 0,
};

export const getContainer = (globalId) => containerRefs[globalId] && containerRefs[globalId].current;
export const getSubContainer = (globalId) => {
  const subContainerDetail = getSubContainerDetail(globalId);
  if (subContainerDetail) {
    return subContainerDetail.element;
  }
};
export const getSubContainerDetail = (globalId) => subContainerDetailRefs[globalId] && subContainerDetailRefs[globalId].current;
export const getSubContainerDetailLeft = (globalId) => subContainerDetailRefs[globalId] && subContainerDetailRefs[globalId].current.left;
export const getSubContainerDetailTop = (globalId) => subContainerDetailRefs[globalId] && subContainerDetailRefs[globalId].current.top;
export const getSubContainerDetailRef = (globalId) => subContainerDetailRefs[globalId];

export const setContainers = (globalId, container, subContainer) => {
  if (globalId !== null) {
    containerRefs[globalId] = {
      current: container,
    };

    const subContainerDetail = {
      ...defaultSubContainerDetail,
      element: subContainer,
    };
    subContainerDetailRefs[globalId] = {
      current: subContainerDetail,
    };

    if (subContainer) {
      subContainer.style.left = "0px";
      subContainer.style.top = "0px";
      subContainer.style.width = "100%";
      subContainer.style.height = "1500px";
      const subContainerRect = subContainer.getBoundingClientRect();
      subContainer.style.width = `${subContainerRect.width + 20}px`;
      subContainer.style.height = `${subContainerRect.height + 20}px`;
      subContainerDetail.width = subContainerRect.width + 20;
      subContainerDetail.height = subContainerRect.height + 20;
      subContainerDetail.initWidth = subContainerRect.width + 20;
      subContainerDetail.initHeight = subContainerRect.height + 20;
    }
  }
};

export const getCanvasRadius = (globalId) => canvasRadiusRefs[globalId] && canvasRadiusRefs[globalId].current;
const setCanvasRadius = (globalId, canvasRadius) => {
  if (canvasRadiusRefs[globalId]) {
    canvasRadiusRefs[globalId].current = canvasRadius;
  } else {
    canvasRadiusRefs[globalId] = { current: canvasRadius };
  }
};

export const getNewCanvas = (globalId, canvasRadius, centerX, centerY) => {
  const container = getContainer(globalId);
  const subContainerDetail = getSubContainerDetail(globalId);

  if (!container || !subContainerDetail.element) {
    return;
  }

  const canvas = document.createElement("canvas");
  subContainerDetail.element.append(canvas);

  const containerRect = container.getBoundingClientRect();

  const xScale = containerRect.width / (containerRect.right - containerRect.left);
  const yScale = containerRect.height / (containerRect.bottom - containerRect.top);

  const clientX = (centerX + subContainerDetail.initWidth / 2) / xScale;
  const clientY = (centerY + subContainerDetail.initHeight / 2) / yScale;

  const left = clientX - canvasRadius;
  const top = clientY - canvasRadius;

  canvas.width = 2 * canvasRadius;
  canvas.height = 2 * canvasRadius;
  canvas.style = `left: ${left}px; top: ${top}px;`;

  const context = canvas.getContext("2d");
  if (context) {
    context.setTransform(1, 0, 0, 1, canvasRadius - centerX, canvasRadius - centerY);
  }

  setCanvasRadius(globalId, canvasRadius);

  return canvas;
};

export const onScroll = (globalId) => {
  const container = getContainer(globalId);
  const subContainerDetail = getSubContainerDetail(globalId);
  if (!container || !subContainerDetail.element) {
    return;
  }

  const canvasDiameter = 2 * getCanvasRadius(globalId);

  if (container.scrollLeft < canvasDiameter) {
    subContainerDetail.left += canvasDiameter;
    subContainerDetail.width += canvasDiameter;
    subContainerDetail.element.style.left = `${subContainerDetail.left}px`;
    container.scrollLeft += canvasDiameter;
  }

  if (container.scrollTop < canvasDiameter) {
    subContainerDetail.top += canvasDiameter;
    subContainerDetail.height += canvasDiameter;
    subContainerDetail.element.style.top = `${subContainerDetail.top}px`;
    container.scrollTop += canvasDiameter;
  }

  if (subContainerDetail.right - (container.scrollLeft - subContainerDetail.left) < canvasDiameter) {
    subContainerDetail.right += canvasDiameter;
    subContainerDetail.width += canvasDiameter;
    subContainerDetail.element.style.width = `${subContainerDetail.width}px`;
  }

  if (subContainerDetail.bottom - (container.scrollTop - subContainerDetail.top) < canvasDiameter) {
    subContainerDetail.bottom += canvasDiameter;
    subContainerDetail.height += canvasDiameter;
    subContainerDetail.element.style.height = `${subContainerDetail.height}px`;
  }
};
