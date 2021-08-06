import React, { useEffect, useState } from "react";
import { verify } from "client";
import { useParams } from "react-router";

type Params = {
  verifcationCode: string;
};

export const Verify = () => {
  const { verifcationCode } = useParams<Params>();
  const [result, setResult] = useState<boolean | null>(null);

  useEffect(
    () => {
      (async () => {
        try {
          setResult(await verify(verifcationCode));
        } catch (err) {
          setResult(false);
        }
      })();
    },
    [verifcationCode],
  );

  return result === null
    ? (<>Verifying...</>)
    : (
      <>
        {result ? "Success!" : "Unknown verification code."}
      </>
    );
};
