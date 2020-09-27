import React from "react";
import styled from "styled-components";

const Key = styled.div`
  border: solid 1px #000;
  width: 4.16%;
  height: 100%;
  background-color: #e1552c;
  :hover {
    opacity: 70%;
  }
`;

const KeyBed = styled.div`
  position: absolute;
  bottom: 0;
  left: 0;
  flex-shrink: 0;
  width: 100%;
  height: 400px;
  display: flex;
  flex-direction: row;
  background-color: #fff;
`;

interface KeyboardProps {}

export const Keyboard = (props: KeyboardProps) => {
  return (
    <KeyBed>
      {Array.from({ length: 24 }, (x, i) => i).map((i) => {
        return <Key></Key>;
      })}
    </KeyBed>
  );
};
