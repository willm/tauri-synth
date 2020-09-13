import React from "react";

interface ButtonProps {
  children: React.ReactNode;
  onClick: () => Promise<void>;
}

export const Button = (props: ButtonProps) => {
  return <button onClick={props.onClick}>{props.children}</button>;
};
