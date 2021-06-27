import React from "react";
import { Tilings } from "./tilings";
import * as client from "client";

// @ts-ignore
global.client = Object.fromEntries(
  Object.entries(client)
    .map(([key, fn]: [string, Function]) => [
      key,
      async (...args) => console.log(await fn(...args)),
    ])
);

export const App = () => (
  <Tilings />
);
