import { lazy } from "react";
import { useIsSignedIn } from "app/account";
import { makeRoutes } from "utils";

export const routes = {
  "sign-up": {
    Component: lazy(() => import("./sign-up").then(({ SignUp }) => ({ default: SignUp }))),
    useDiscoverable: () => !useIsSignedIn(),
  },
};

export const Routes = makeRoutes(routes);
