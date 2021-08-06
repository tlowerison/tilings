import React, { useMemo } from "react";
import styles from "./styles.module.scss";
import { Breadcrumbs as MuiBreadcrumbs, Typography } from "@material-ui/core";
import { findInRoutes, routerBasename } from "utils";
import { routes as allRoutes } from "app/nav/routes";
import { useLocation } from "react-router";

type Props = {
  maxItems: number;
};

export const Breadcrumbs = ({ maxItems }: Props) => {
  const { pathname } = useLocation();

  const subPathnames = useMemo(
    () => `${routerBasename}${pathname}`.slice(1).split("/").filter(e => e !== ""),
    [pathname],
  );

  const isValidRoute = useMemo(
    () => Boolean(findInRoutes(subPathnames, allRoutes)),
    [subPathnames],
  );

  if (!isValidRoute) {
    return (
      <div className={styles.wrapper} />
    );
  }

  return (
    <div className={styles.wrapper}>
      <MuiBreadcrumbs maxItems={maxItems} aria-label="breadcrumb">
        <a style={{ color: "inherit" }} href={window.location.origin}>
          <Typography variant="h6">
            Home
          </Typography>
        </a>
        {subPathnames.map((_, pathIndex) => findInRoutes(subPathnames.slice(0, pathIndex + 1), allRoutes))}
      </MuiBreadcrumbs>
    </div>
  );
};
