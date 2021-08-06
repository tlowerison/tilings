import { lazy } from "react";
import { isNotDiscoverable, makeRoutes } from "utils";

export const routes = {
  "sign-out": {
    Component: lazy(() => import("./sign-out").then(({ SignOut }) => ({ default: SignOut }))),
    useDiscoverable: isNotDiscoverable,
  },
};

export const Routes = makeRoutes(routes);
