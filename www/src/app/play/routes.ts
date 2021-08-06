import { lazy } from "react";
import { isDiscoverable, makeRoutes } from "utils";

export const routes = {
  "play": {
    Component: lazy(() => import("./play").then(({ Play }) => ({ default: Play }))),
    useDiscoverable: isDiscoverable,
  },
};

export const Routes = makeRoutes(routes);
