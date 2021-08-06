import React, { useEffect, useState } from "react";
import styles from "./styles.module.scss";
import { Tilings } from "components";
import { Typography } from "@material-ui/core";
import { routerBasename } from "utils";
import { tilingSearch } from "client";

const thumbnailSize = 300;

export const Home = () => {
  const [tilings, setTilings] = useState([]);

  useEffect(() => {
    (async () => {
      try {
        setTilings((await tilingSearch(".")).map(({ id, title }) => ({ tiling: { id, title } })));
      } catch (e) {
        console.log(e);
      }
    })();
  }, []);

  // remove trailing slash after basename
  if (window.location.pathname !== routerBasename) {
    window.history.pushState(null, "Tilings", routerBasename);
  }

  return (
    <div className={styles.container}>
      <Typography variant="h3">Tilings</Typography>
      <p>My work-in-progress catalog of all uniform tilings :)</p>
      <Tilings
        minRows={3}
        thumbnailSize={thumbnailSize}
        tilings={tilings}
      />
    </div>
  );
};
