import React, { useState, useEffect } from "react";
import "./App.css";
import { Keyboard } from "./components/Keyboard";
import * as tauriEvent from "tauri/api/event";
import styled from "styled-components";
if (!(window as any).__TAURI_INVOKE_HANDLER__) {
  (window as any).__TAURI_INVOKE_HANDLER__ = () => {};
}

interface NoteOnEvent {
  payload: {
    note: number;
  };
}

const Slider = styled.input`
  transform: rotate(-90deg);
  height: 70%;
  margin-top: 20px;
`;

interface FaderProps {
  id: string;
  onChange: (event: React.ChangeEvent<HTMLInputElement>) => void;
  name: string;
}

const FaderContainer = styled.div`
  width: 19%;
`;

function Fader(props: FaderProps) {
  return (
    <FaderContainer>
      <Slider
        type="range"
        min="0"
        max="1"
        step="0.01"
        onChange={props.onChange}
        id={props.id}
      ></Slider>
      <label style={{ width: "100%", height: "10%" }} htmlFor={props.id}>
        {props.name}
      </label>
    </FaderContainer>
  );
}

function App() {
  const [midiNote, setMidiNote] = useState<number>();
  const [loaded, setLoaded] = useState<boolean>(false);
  useEffect(() => {
    if (tauriEvent && tauriEvent.listen) {
      tauriEvent.listen("ready", (evt) => {
        setLoaded(true);
      });
      tauriEvent.listen("message", (evt: NoteOnEvent) => {
        setMidiNote(evt.payload.note);
      });
    }
  }, [midiNote, loaded]);
  return (
    <div className="App">
      <header className="App-header">
        <div style={{ flexGrow: 1, display: "flex" }}>
          {loaded ? (
            <div style={{ height: "200px", display: "flex" }}>
              <Fader
                onChange={(e) => {
                  tauriEvent.emit("attack", e.target.value);
                  console.log(e.target.value);
                }}
                id="attack"
                name="attack"
              ></Fader>
              <Fader
                onChange={(e) => {
                  tauriEvent.emit("decay", e.target.value);
                  console.log(e.target.value);
                }}
                id="decay"
                name="decay"
              ></Fader>
              <Fader
                onChange={(e) => {
                  tauriEvent.emit("sustain", e.target.value);
                  console.log(e.target.value);
                }}
                id="sustain"
                name="sustain"
              ></Fader>
              <Fader
                onChange={(e) => {
                  tauriEvent.emit("release", e.target.value);
                  console.log(e.target.value);
                }}
                id="release"
                name="release"
              ></Fader>
              <Fader
                onChange={(e) => {
                  tauriEvent.emit("delay_wet", e.target.value);
                  console.log(e.target.value);
                }}
                id="delay-wet"
                name="delay wet"
              ></Fader>
              <Keyboard selectedKey={midiNote}></Keyboard>
            </div>
          ) : (
            <h2>Loading ...</h2>
          )}
        </div>
      </header>
    </div>
  );
}

export default App;
