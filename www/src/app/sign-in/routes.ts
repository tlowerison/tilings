import { lazy } from "react";
import { useIsSignedIn } from "app/account";
import { makeRoutes } from "utils";

export const routes = {
  "sign-in": {
    Component: lazy(() => import("./sign-in").then(({ SignIn }) => ({ default: SignIn }))),
    useDiscoverable: () => !useIsSignedIn(),
  },
};

export const Routes = makeRoutes(routes);
