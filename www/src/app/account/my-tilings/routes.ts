import { lazy } from "react";
import { isDiscoverable } from "utils";

export const myTilingsRoutes = {
  "my-tilings": {
    Component: lazy(() => import("./my-tilings").then(({ MyTilings }) => ({ default: MyTilings }))),
    useDiscoverable: isDiscoverable,
  },
};
