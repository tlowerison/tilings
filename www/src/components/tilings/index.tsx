import React, { useEffect, useMemo, useRef, useState } from "react";
import styles from "./styles.module.scss";
import { Link } from "react-router-dom";
import { LinkItem, Thumbnail } from "components";
import { Typography } from "@material-ui/core";
import { getAccountTilings } from "client";
import { setRootStyle } from "utils";

type Props = {
  minRows: number;
  thumbnailSize: number;
  tilings: {
    tiling: {
      id: number;
      title: string;
    };
  }[];
};

export const Tilings = ({ minRows, thumbnailSize, tilings }: Props) => {
  const ref = useRef<HTMLDivElement>(null);

  const [numRowsDisplayed, setNumRowsDisplayed] = useState(minRows);
  const thumbnailContainerSize = useMemo(() => thumbnailSize + 4 + 2 * 12, [thumbnailSize]);
  const numThumbnailsPerRow = useMemo(
    () => Math.floor(window.outerWidth / thumbnailContainerSize),
    [thumbnailContainerSize],
  );

  useEffect(
    () => {
      const html = document.documentElement;
      const htmlRect = html.getBoundingClientRect();
      const updateNumRowsDisplayed = () => {
        let additionalNumRowsDisplayed = 0;
         while (
           htmlRect.height - window.outerHeight - html.scrollTop + additionalNumRowsDisplayed * thumbnailContainerSize < thumbnailContainerSize &&
           additionalNumRowsDisplayed < minRows &&
           numRowsDisplayed * numThumbnailsPerRow < tilings.length
         ) {
           additionalNumRowsDisplayed += 1;
         }
         setNumRowsDisplayed(numRowsDisplayed + additionalNumRowsDisplayed);
      };
      updateNumRowsDisplayed();
      document.body.onscroll = updateNumRowsDisplayed;
    },
    [numRowsDisplayed, setNumRowsDisplayed, tilings],
  );

  useEffect(() => {
    setRootStyle("overflow-y", "scroll");
    return () => {
      setRootStyle("overflow-y", "hidden");
      document.body.onscroll = null;
    };
  }, []);

  return (
    <div className={styles.container} ref={ref}>
      {tilings.slice(0, numRowsDisplayed * numThumbnailsPerRow).map(({ tiling: { title, id } }) => (
        <div key={id} className={styles.item}>
          <Link to={`/play#${id}`}>
            <Thumbnail
              height={thumbnailSize}
              tilingId={id}
              width={thumbnailSize}
            />
          </Link>
          <Typography variant="h5">
            <LinkItem
              title={title}
              to={`/play#${id}`}
            />
          </Typography>
        </div>
      ))}
    </div>
  );
};
