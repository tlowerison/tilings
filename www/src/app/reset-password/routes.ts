import { lazy } from "react";
import { useIsSignedIn } from "app/account";
import { makeRoutes } from "utils";

export const routes = {
  "reset-password/:passwordResetCode": {
    Component: lazy(() => import("./reset-password").then(({ ResetPassword }) => ({ default: ResetPassword }))),
    useDiscoverable: () => !useIsSignedIn(),
  },
};

export const Routes = makeRoutes(routes);
