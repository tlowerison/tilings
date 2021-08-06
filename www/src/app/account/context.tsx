import { createContext } from "react";

type ContextType = [
  Account | null,
  (account: Account | null | ((account: Account | null) => Account | null)) => void,
  boolean, // loadingAccount
];

export const Context = createContext<ContextType>([
  null,
  () => {},
  false,
]);
