import { Context } from "./context";
import { useContext } from "react";

export const useIsSignedIn = () => {
  const [account] = useContext(Context);
  return Boolean(account);
};
