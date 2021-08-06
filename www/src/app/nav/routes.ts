import { lazy } from "react";
import { isDiscoverable, routerBasename } from "utils";
import { routes as accountRoutes } from "app/account/routes";
import { routes as playRoutes } from "app/play/routes";
import { routes as randomRoutes } from "app/random/routes";
import { routes as signInRoutes } from "app/sign-in/routes";

export const routes = {
  [routerBasename.slice(1)]: {
    Component: lazy(() => import("app/home").then(({ Home }) => ({ default: Home }))),
    useDiscoverable: isDiscoverable,
    routes: {
      ...accountRoutes,
      ...playRoutes,
      ...randomRoutes,
      ...signInRoutes,
    },
  },
};
