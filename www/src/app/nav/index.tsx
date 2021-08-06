import React from "react";
import { Desktop } from "./desktop";
import { Mobile } from "./mobile";
import { isMobile } from "utils";
import { useLocation } from "react-router";

export const Nav = () => {
  if (["/sign-in", "/sign-up"].includes(useLocation().pathname)) {
    return null;
  }

  return (
    isMobile
      ? <Mobile />
      : <Desktop />
  );
};
