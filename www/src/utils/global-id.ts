let globalId = 0;

export const newGlobalId = () => {
  globalId += 1;
  return globalId - 1;
};
