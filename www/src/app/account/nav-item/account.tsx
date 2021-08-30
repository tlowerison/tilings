import React, { createRef, useContext } from "react";
import { Context } from "app/account/context";
import { Dropdown, LinkItem, MenuLinkItem } from "components";
import { MenuItem } from "@material-ui/core";
import { NavItemProps } from "app/nav/item-props";
import { isMobile } from "utils";

export const Account = ({ ref, clearAnchorEl }: NavItemProps) => {
  const [account] = useContext(Context);

  if (!account) {
    return null;
  }

  return isMobile
    ? (
      <>
        {[
          ({ ref }) => (
            <MenuLinkItem
              ref={ref}
              title={account.displayName}
              titleCase={false}
              to="/account"
            />
          ),
          ({ ref }) => (
            <MenuLinkItem
              ref={ref}
              to="/account/my-tilings"
              title="My Tilings"
            />
          ),
          ({ ref }) => (
            <MenuLinkItem
              ref={ref}
              to="/sign-out"
              title="Sign Out"
            />
          ),
        ].map(Item => {
          const ref = createRef<HTMLElement>();
          return (
            <MenuItem onClick={() => {
              ref.current?.click?.();
              // @ts-ignore
              ref.current?.base?.click?.();
              clearAnchorEl?.();
            }}>
              <Item ref={ref} />
            </MenuItem>
          );
        })}
      </>
    ) : (
      <Dropdown
        Title={() => (
          <LinkItem
            ref={ref}
            title={account.displayName}
            titleCase={false}
            to="/account"
          />
        )}
        Items={[
          ({ ref }) => (
            <MenuLinkItem
              ref={ref}
              to="/account/my-tilings"
              title="My Tilings"
            />
          ),
          ({ ref }) => (
            <MenuLinkItem
              ref={ref}
              to="/sign-out"
              title="Sign Out"
            />
          ),
        ]}
      />
    );
};
