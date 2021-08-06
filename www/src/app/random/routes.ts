import { lazy } from "react";
import { isDiscoverable, makeRoutes } from "utils";

export const routes = {
  "random": {
    Component: lazy(() => import("./random").then(({ Random }) => ({ default: Random }))),
    useDiscoverable: isDiscoverable,
  },
};

export const Routes = makeRoutes(routes);
