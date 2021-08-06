import { RefObject } from "react";

export type NavItemProps = {
  clearAnchorEl?: () => void;
  ref?: RefObject<HTMLElement>;
};
