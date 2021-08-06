import React from "react";
import styles from "./styles.module.scss";
import { Help as HelpIcon } from "@material-ui/icons";
import { LinkItem } from "components";
import { NavItemProps } from "app/nav/item-props";
import { Tooltip } from "@material-ui/core";
import { useLocation } from "react-router";
import { useIsSignedIn } from "app/account/use-is-signed-in";

export const SignIn = ({ ref }: NavItemProps) => {
  const { pathname } = useLocation();
  const isSignedIn = useIsSignedIn();

  if (isSignedIn || ["/sign-in", "/sign-up"].includes(pathname)) {
    return null;
  }

  return (
    <Tooltip title="Sign up to contribute new tilings" placement="bottom-start">
      <div className={styles.signInSignUpContainer}>
        <LinkItem
          ref={ref}
          title="Sign In / Up"
          to="/sign-in"
        />
        <HelpIcon />
      </div>
    </Tooltip>
  );
};
