type ArrayValue <T extends ReadonlyArray<unknown>> =
  T extends ReadonlyArray<infer ArrayValue>
    ? ArrayValue
    : never;
