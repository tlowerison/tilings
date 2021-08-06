import React, { useCallback, useState } from "react";
import styles from "./styles.module.scss";
import { Breadcrumbs } from "app/nav/breadcrumbs";
import { Fade, IconButton, Menu } from "@material-ui/core";
import { Menu as MenuIcon } from "@material-ui/icons";
import { items } from "app/nav/items";

export const Mobile = () => {
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);

  const clearAnchorEl = useCallback(() => setAnchorEl(null), [setAnchorEl]);

  return (
    <nav className={styles.nav}>
      <Breadcrumbs maxItems={3} />
      <div className={styles.right}>
        <IconButton
          classes={{ root: styles.iconButton }}
          onClick={(event: React.MouseEvent<HTMLElement>) => setAnchorEl(event.currentTarget)}
        >
          <MenuIcon />
        </IconButton>
      </div>
      <Menu
        anchorEl={anchorEl}
        keepMounted
        open={Boolean(anchorEl)}
        onClose={clearAnchorEl}
        TransitionComponent={Fade}
      >
        {items.map((Item, i) => (
          <Item
            key={i}
            clearAnchorEl={clearAnchorEl}
          />
        ))}
      </Menu>
    </nav>
  );
};
