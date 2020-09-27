import React from "react";
import styled, { css } from "styled-components";

interface KeyProps {
  selected?: boolean;
}

const Key = styled.div<KeyProps>`
  border: solid 1px #000;
  width: 4.16%;
  height: 100%;
  background-color: #e1552c;

  :hover {
    opacity: 70%;
  }
  ${(props) =>
    props.selected &&
    css`
      opacity: 70%;
    `}
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

interface KeyboardProps {
  selectedKey?: number;
}

export const Keyboard = (props: KeyboardProps) => {
  return (
    <KeyBed>
      {Array.from({ length: 24 }, (_, i) => i).map((i) => {
        // first key is middle c (midi note 60)
        const midi_note = i + 60;
        return (
          <Key key={midi_note} selected={props.selectedKey === midi_note}></Key>
        );
      })}
    </KeyBed>
  );
};
