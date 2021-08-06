import { makeRoutes } from "utils";
import { myTilingsRoutes } from "./my-tilings";
import { verifyRoutes } from "./verify";
import { useIsSignedIn } from "./use-is-signed-in";

export const routes = {
  "account": {
    useDiscoverable: useIsSignedIn,
    routes: {
      ...myTilingsRoutes,
      ...verifyRoutes,
    },
  },
};

export const Routes = makeRoutes(routes);
