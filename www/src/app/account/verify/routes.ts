import { lazy } from "react";
import { isNotDiscoverable } from "utils";

export const verifyRoutes = {
  "verify/:verificationCode": {
    Component: lazy(() => import("./verify").then(({ Verify }) => ({ default: Verify }))),
    useDiscoverable: isNotDiscoverable,
  }
};
