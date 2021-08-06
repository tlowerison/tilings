import React, { ComponentProps } from "react";
import { Typography } from "@material-ui/core";

type ItemProps = {
  title: string;
  titleCase?: boolean;
  variant?: ComponentProps<typeof Typography>["variant"] | "p" | "span";
};

const toTitleCase = (dashCase: string) => dashCase
  .split("-")
  .map(word => `${word[0].toUpperCase()}${word.slice(1)}`)
  .join(" ");

export const Item = ({ title, titleCase = true, variant = "h6" }: ItemProps) => {
  const text = titleCase ? toTitleCase(title) : title;

  switch (variant) {
    case "p": return (<p>{text}</p>);
    case "span": return (<span>{text}</span>);
    default: return (
      <Typography variant={variant}>
        {titleCase ? toTitleCase(title) : title}
      </Typography>
    )
  }
};
