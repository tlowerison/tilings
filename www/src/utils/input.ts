import { KeyboardEvent } from "react";

export const catchFormSubmitOnEnter = (...callbacks: ((event?: KeyboardEvent<HTMLInputElement>) => void)[]) =>
  (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === "Enter") {
      for (const callback of callbacks) {
        callback(event);
      }
    }
  };

export const debounce = (fn: Function, timer = 300) => {
  let timeout: NodeJS.Timeout;
  return (...args: any[]) => {
    clearTimeout(timeout);
    timeout = setTimeout(() => fn.apply(this, args), timer);
  };
};
