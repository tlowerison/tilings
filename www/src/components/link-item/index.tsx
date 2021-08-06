import React, { ComponentProps, RefObject } from "react";
import styles from "./styles.module.scss";
import { Item } from "components/item";
import { Link } from "react-router-dom";
import { MenuItem } from "@material-ui/core";

type LinkItemProps = ComponentProps<typeof Item> & {
  ref?: RefObject<any>;
  to: string;
};

export const LinkItem = ({ ref, to, ...itemProps }: LinkItemProps) => (
  <Link className={styles.link} ref={ref} href="#" to={to}>
    <Item {...itemProps} />
  </Link>
);

export const MenuLinkItem = ({ ref, to, ...itemProps }: LinkItemProps) => (
  <Link className={styles.link} ref={ref} href="#" to={to}>
    <MenuItem>
      <Item {...itemProps} />
    </MenuItem>
  </Link>
);
