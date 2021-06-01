import "regenerator-runtime/runtime.js";
import React, { ReactElement } from "react";
import ReactDOM from "react-dom";
import styles from "./styles.module.scss";

(async () => {
  const { App } = await import("app");
  const root = document.getElementById("root");
  root && ReactDOM.render(<div className={styles.app}><App /></div> as ReactElement, root);
})();
