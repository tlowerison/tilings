import React, { Suspense, useEffect, useState } from "react";
// import * as client from "client";
import styles from "./styles.module.scss";
import { Context as AccountContext, Routes as AccountRoutes } from "./account";
import { Home } from "./home";
import { Nav } from "./nav";
import { Route } from "react-router-dom";
import { Routes as PlayRoutes } from "./play";
import { Routes as RandomRoutes } from "./random";
import { Routes as ResetPasswordRoutes } from "./reset-password";
import { Routes as SignInRoutes } from "./sign-in";
import { Routes as SignOutRoutes } from "./sign-out";
import { Routes as SignUpRoutes } from "./sign-up";
import { getAccount } from "client";

export const App = () => {
  const [account, setAccount] = useState<Account | null>(null);
  const [loadingAccount, setLoadingAccount] = useState(true);

  useEffect(
    () => {
      (async () => {
        try { setAccount(await getAccount()) } catch (e) {}
        setLoadingAccount(false);
      })();
    },
    [],
  );

  return (
    <AccountContext.Provider value={[account, setAccount, loadingAccount]}>
      <div className={styles.container}>
        <Nav />
        <Suspense fallback={null}>
          <Route exact path="/">
            <Home />
          </Route>
          {account && <AccountRoutes />}
          <PlayRoutes />
          <RandomRoutes />
          {!account && <ResetPasswordRoutes />}
          {!account && <SignInRoutes />}
          {account && <SignOutRoutes />}
          {!account && <SignUpRoutes />}
        </Suspense>
      </div>
    </AccountContext.Provider>
  );
};

// @ts-ignore
// global.client = Object.fromEntries(
//   Object.entries(client)
//     .map(([key, fn]: [string, Function]) => [
//       key,
//       async (...args) => {
//         try {
//           console.log(await fn(...args));
//         } catch (err) {
//           console.error(err);
//         }
//       },
//     ])
// );
