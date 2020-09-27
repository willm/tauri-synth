import React, { useState, useEffect } from "react";
import "./App.css";
import { Button } from "./components/Button";
import { Keyboard } from "./components/Keyboard";
import * as tauriEvent from "tauri/api/event";
if (!(window as any).__TAURI_INVOKE_HANDLER__) {
  (window as any).__TAURI_INVOKE_HANDLER__ = () => {};
}

function App() {
  const [message, setMessage] = useState<string>("");
  useEffect(() => {
    if (tauriEvent && tauriEvent.listen) {
      tauriEvent.listen("message", (evt) => {
        setMessage(evt.payload as string);
      });
    }
  }, [message]);
  return (
    <div className="App">
      <header className="App-header">
        <div style={{ flexGrow: 1 }}>
          <h2>{message}</h2>
          <Button
            onClick={async () => {
              if (tauriEvent && tauriEvent.listen) {
                tauriEvent.listen("message", (evt) => {
                  setMessage(evt.payload as string);
                });
                tauriEvent.emit("start-synth", "start please");
              }
            }}
          >
            Do it
          </Button>
          <Keyboard></Keyboard>
        </div>
      </header>
    </div>
  );
}

export default App;
