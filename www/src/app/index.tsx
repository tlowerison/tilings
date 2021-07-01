import React from "react";
import { Tilings } from "./tilings";
import { Route, Switch } from "react-router-dom";
import { Verify } from "./verify";
import * as client from "client";

export const App = () => (
  <Switch>
    <Route path="/atlas">
      <Tilings />
    </Route>
    <Route path="/verify/:verifcationCode">
      <Verify />
    </Route>
  </Switch>
);

// @ts-ignore
global.client = Object.fromEntries(
  Object.entries(client)
    .map(([key, fn]: [string, Function]) => [
      key,
      async (...args) => {
        try {
          console.log(await fn(...args));
        } catch (err) {
          console.error(err);
        }
      },
    ])
);
