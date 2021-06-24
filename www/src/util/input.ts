export const debounce = (fn: Function, timeout = 300) => {
  let timer: NodeJS.Timeout;
  return (...args: any[]) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn.apply(this, args), timeout);
  };
};
