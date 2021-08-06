import React, { useEffect, useState } from "react";
import { Tilings } from "components";
import { getAccountTilings } from "client";

const thumbnailSize = 300;

export const MyTilings = () => {
  const [accountTilings, setAccountTilings] = useState([]);

  useEffect(() => {
    (async () => {
      try {
        setAccountTilings(await getAccountTilings());
      } catch (e) {
        console.log(e);
      }
    })();
  }, []);

  return (
    <Tilings
      minRows={1}
      thumbnailSize={thumbnailSize}
      tilings={accountTilings}
    />
  );
};
