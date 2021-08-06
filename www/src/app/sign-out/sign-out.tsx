import React, { useContext, useEffect } from "react";
import { Context as AccountContext } from "app/account";
import { signOut } from "client";
import { useHistory } from "react-router";

export const SignOut = () => {
  const history = useHistory();

  const [_, setAccount] = useContext(AccountContext);

  useEffect(() => {
    (async () => {
      setAccount(null);
      await signOut();
      history.push("/");
    })();
  }, []);

  return null;
};
