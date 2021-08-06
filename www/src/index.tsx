import "regenerator-runtime/runtime.js";
import "./styles.module.scss";
import React, { ReactElement } from "react";
import ReactDOM from "react-dom";
import { BrowserRouter as Router, Switch } from "react-router-dom";
import { ThemeProvider, createMuiTheme } from "@material-ui/core/styles";
import { init } from "client";
import { routerBasename } from "utils";

init();

const muiTheme = createMuiTheme({
  typography: {
    fontFamily: "monospace",
  },
  props: {
    MuiButtonBase: {
      disableRipple: true,
    },
  },
});

(async () => {
  const { App } = await import("app");
  const root = document.getElementById("root");
  root && ReactDOM.render(
    (
      <Router basename={routerBasename}>
        <ThemeProvider theme={muiTheme}>
          <Switch>
            <App />
          </Switch>
        </ThemeProvider>
      </Router>
    ) as ReactElement,
    root,
  );
})();
