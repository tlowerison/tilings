import React, { createRef } from "react";
import styles from "./styles.module.scss";
import { Breadcrumbs } from "app/nav/breadcrumbs";
import { items } from "app/nav/items";

const itemPairs = items.map(Item => [Item, createRef<any>()] as const);

export const Desktop = () => {
  return (
    <nav className={styles.nav}>
      <Breadcrumbs maxItems={6} />
      <div className={styles.items}>
        {itemPairs.map(([Item, ref]) => (
          <div className={styles.item}>
            <Item ref={ref} />
          </div>
        ))}
      </div>
    </nav>
  );
};
