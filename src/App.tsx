import React, { useState, useEffect } from "react";
import "./App.css";
import { Keyboard } from "./components/Keyboard";
import * as tauriEvent from "tauri/api/event";
if (!(window as any).__TAURI_INVOKE_HANDLER__) {
  (window as any).__TAURI_INVOKE_HANDLER__ = () => {};
}

interface NoteOnEvent {
  payload: {
    note: number;
  };
}

function App() {
  const [midiNote, setMidiNote] = useState<number>();
  const [loaded, setLoaded] = useState<boolean>(false);
  useEffect(() => {
    if (tauriEvent && tauriEvent.listen) {
      tauriEvent.listen("message", (evt: NoteOnEvent) => {
        setLoaded(true);
        setMidiNote(evt.payload.note);
      });
    }
  }, [midiNote, loaded]);
  return (
    <div className="App">
      <header className="App-header">
        <div style={{ flexGrow: 1 }}>
          {loaded ? (
            <Keyboard selectedKey={midiNote}></Keyboard>
          ) : (
            <h2>Loading ...</h2>
          )}
        </div>
      </header>
    </div>
  );
}

export default App;
