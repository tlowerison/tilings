import "regenerator-runtime/runtime.js";
import React, { ReactElement } from "react";
import ReactDOM from "react-dom";
import { BrowserRouter as Router, Route, Switch } from "react-router-dom";
import styles from "./styles.module.scss";

(async () => {
  const { App } = await import("app");
  const root = document.getElementById("root");
  root && ReactDOM.render(
    <div className={styles.app}>
      <Router>
        <Switch>
          <Route path="/tilings">
            <App />
          </Route>
        </Switch>
      </Router>
    </div> as ReactElement,
    root,
  );
})();
