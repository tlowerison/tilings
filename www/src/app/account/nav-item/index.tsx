import React, { useContext } from "react";
import { Account } from "./account";
import { Context as AccountContext } from "app/account/context";
import { SignIn } from "./sign-in";
import { NavItemProps } from "app/nav/item-props";
import { useIsSignedIn } from "app/account/use-is-signed-in";

export const NavItem = (props: NavItemProps) => {
  const [_account, _setAccount, loadingAccount] = useContext(AccountContext);
  const isSignedIn = useIsSignedIn();

  if (loadingAccount) {
    return null;
  }

  return !isSignedIn
    ? (<SignIn {...props} />)
    : (<Account {...props} />);
};
