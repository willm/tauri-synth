import React, { useState, useEffect } from 'react';
import './App.css';
import { Keyboard } from './components/Keyboard';
import * as tauriEvent from 'tauri/api/event';
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
      tauriEvent.listen('ready', (evt) => {
        setLoaded(true);
      });
      tauriEvent.listen('message', (evt: NoteOnEvent) => {
        setMidiNote(evt.payload.note);
      });
    }
  }, [midiNote, loaded]);
  return (
    <div className="App">
      <header className="App-header">
        <div style={{ flexGrow: 1 }}>
          <input
            type="range"
            min="1"
            max="16"
            step="0.01"
            style={{ backgroundColor: '#22222' }}
            onChange={(e) => {
              tauriEvent.emit('dist_amount', e.target.value);
              console.log(e.target.value);
            }}
          ></input>
          <input
            type="range"
            min="0"
            max="16"
            step="0.01"
            style={{ backgroundColor: '#22222' }}
            onChange={(e) => {
              tauriEvent.emit('fm_amount', e.target.value);
              console.log(e.target.value);
            }}
          ></input>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            style={{ backgroundColor: '#22222' }}
            onChange={(e) => {
              tauriEvent.emit('attack', e.target.value);
              console.log(e.target.value);
            }}
          ></input>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            style={{ backgroundColor: '#22222' }}
            onChange={(e) => {
              tauriEvent.emit('decay', e.target.value);
              console.log(e.target.value);
            }}
          ></input>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            style={{ backgroundColor: '#22222' }}
            onChange={(e) => {
              tauriEvent.emit('sustain', e.target.value);
              console.log(e.target.value);
            }}
          ></input>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            style={{ backgroundColor: '#22222' }}
            onChange={(e) => {
              tauriEvent.emit('release', e.target.value);
              console.log(e.target.value);
            }}
          ></input>
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
