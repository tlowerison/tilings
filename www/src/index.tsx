import "regenerator-runtime/runtime.js";
import React, { ReactElement } from "react";
import ReactDOM from "react-dom";
import styles from "./styles.module.scss";
import { BrowserRouter as Router } from "react-router-dom";
import { ThemeProvider, createMuiTheme } from "@material-ui/core/styles";

const muiTheme = createMuiTheme({
  typography: {
    fontFamily: "monospace",
  },
});

(async () => {
  const { App } = await import("app");
  const root = document.getElementById("root");
  root && ReactDOM.render(
    (
      <div className={styles.app}>
        <Router basename="/tilings">
          <ThemeProvider theme={muiTheme}>
            <App />
          </ThemeProvider>
        </Router>
      </div>
    ) as ReactElement,
    root,
  );
})();
