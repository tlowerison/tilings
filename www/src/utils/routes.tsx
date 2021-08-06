import React, { ReactNode } from "react";
import { Dropdown } from "components";
import { Item, LinkItem, MenuLinkItem } from "components";
import { Route, Switch } from "react-router-dom";

export const routerBasename = "/tilings";

export type Component = React.LazyExoticComponent<() => (JSX.Element | null)>;

export type Routes = {
  Component?: Component;
  useDiscoverable?: () => boolean;
  routes?: Record<string, Routes>;
};

export const isDiscoverable = () => true;
export const isNotDiscoverable = () => false;
const useDefaultDiscoverable = isNotDiscoverable;

export const makeRoutes = (routes: Record<string, Routes>, rootPath: string = "/") => {
  const components = Object.entries(routes).map(([path, config]) => {
    let Components: Component[] = [];
    if ("Component" in config && config.Component) {
      Components.push(config.Component);
    }
    if (config.routes && Object.keys(config.routes).length > 0) {
      // @ts-ignore
      Components.push(makeRoutes(config.routes, `${rootPath}${path}/`));
    }

    if (Components.length === 0) {
      return [path, []] as const;
    }

    return [path, Components] as const;
  });

  return () => (
    <Switch>
      {components.map(([path, Components]) => (
        <Route key={path} path={`${rootPath}${path}`}>
          {Components.map(Component => (<Component key={path} />))}
        </Route>
      ))}
    </Switch>
  );
};

export const findInRoutes = (
  subPathnames: string[],
  routes: Record<string, Routes>,
  index: number = 0,
): ReactNode | false => {
  const subPathname = subPathnames[index];
  if (subPathname in routes) {
    const { Component, routes: subRoutes } = routes[subPathname];

    if (index < subPathnames.length - 1) {
      if (subRoutes) {
        return findInRoutes(subPathnames, subRoutes, index + 1);
      }
      return (<Item title={subPathname} />);
    }

    const to = subPathnames.slice(1, index + 1);

    if (subRoutes && Object.keys(subRoutes).length > 0) {
      return (
        <Dropdown
          Title={Component ? ({ ref }) => (
            <LinkItem
              ref={ref}
              to={to.length > 0 ? `/${to.join("/")}` : ""}
              title={subPathname}
            />
          ) : () => <Item title={subPathname} />}
          Items={
            Object.entries(subRoutes)
              .map(([subPathname, { useDiscoverable = useDefaultDiscoverable }]) => ({ ref }) => {
                const discoverable = useDiscoverable();
                return discoverable ? (
                  <MenuLinkItem
                    ref={ref}
                    to={`/${[...to, subPathname].join("/")}`}
                    title={subPathname}
                  />
                ) : null;
              })
          }
        />
      );
    }

    if (Component) {
      return (
        <LinkItem
          to={`/${to.join("/")}`}
          title={subPathname}
        />
      );
    }

    return (<Item title={subPathname} />);
  }
  return false;
};
