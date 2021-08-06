import { isMobile } from "./device";

export const drawerWidth = isMobile ? window.outerWidth : 360;

export const setRootStyle = (key: string, value: string) => {
  document.documentElement.style[key] = value;
  document.body.style[key] = value;
};
