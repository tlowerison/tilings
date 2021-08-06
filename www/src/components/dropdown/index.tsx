import React, { RefObject, createRef, forwardRef, useMemo, useState } from "react";
import styles from "./styles.module.scss";
import { ArrowDropDown as ArrowDropDownIcon } from "@material-ui/icons";
import { Fade, Menu } from "@material-ui/core";

type Props = {
  Title: (props: { ref?: RefObject<any> }) => (JSX.Element | null);
  Items: ((props: { ref?: RefObject<any> }) => (JSX.Element | null))[];
};

// @ts-ignore
export const Dropdown = forwardRef<HTMLElement, Props>(({ Title, Items }, ref) => {
  const [anchorEl, setAnchorEl] = useState<HTMLElement | null>(null);
  const titleRef = useMemo(() => createRef<any>(), [Title]);
  const itemPairs = useMemo(() => Items.map(Item => [Item, createRef<any>()] as const), [Items]);

  return (
    <>
      <div
        ref={ref as React.RefObject<HTMLDivElement>}
        className={styles.titleContainer}
      >
        <Title ref={titleRef} />
        <div
          className={styles.dropdownIconContainer}
          onClick={
            // @ts-ignore
            (event: Event) => setAnchorEl(event.currentTarget)
          }
        >
          <ArrowDropDownIcon />
        </div>
      </div>
      <Menu
        anchorEl={anchorEl}
        anchorOrigin={{
          vertical: "bottom",
          horizontal: "left",
        }}
        getContentAnchorEl={null}
        keepMounted
        open={Boolean(anchorEl)}
        onClose={() => setAnchorEl(null)}
        TransitionComponent={Fade}
      >
        {itemPairs.map(([Item, itemRef]) => (
          // require consumer to pass MenuItems in order to enable optional rendering of MenuItems at render time
          <div onClick={() => setAnchorEl(null)}>
            <Item ref={itemRef} />
          </div>
        ))}
      </Menu>
    </>
  );
});
