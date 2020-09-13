import React, { useState, useEffect } from "react";
import logo from "./logo.svg";
import "./App.css";
import { Button } from "./components/Button";
import { promisified } from "tauri/api/tauri";
import { listen, emit } from "tauri/api/event";

function App() {
  const [message, setMessage] = useState<string>("");
  useEffect(() => {
    listen("message", (evt) => {
      setMessage(evt.payload as string);
    });
  }, [message]);
  return (
    <div className="App">
      <header className="App-header">
        <img src={logo} className="App-logo" alt="logo" />
        <h2>{message}</h2>
        <Button
          onClick={async () => {
            listen("message", (evt) => {
              setMessage(evt.payload as string);
            });
            emit("start-synth", "start please");
            /*const result: string = await promisified({
              cmd: "startSynth",
              argument: "Hi Rust",
            });
            setMessage(result);*/
          }}
        >
          Do it
        </Button>
      </header>
    </div>
  );
}

export default App;
