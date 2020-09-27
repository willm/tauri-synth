#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod cmd;
use app::control::midi;
use app::core;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use tauri::WebviewMut;

fn handle_note_on(synth_sender: &Sender<[f32; 3]>, wv_clone: &mut WebviewMut, note: u8) {
  let freq = midi::midi_to_freq(note);
  synth_sender.send([freq, freq + 3.0, freq - 1.0]).unwrap();
  tauri::event::emit(
    wv_clone,
    String::from("message"),
    Some(format!("NOTE ON midi note {} {}Hz", note, freq)),
  )
  .unwrap();
}

fn main() {
  tauri::AppBuilder::new()
    .setup(move |webview, _source| {
      // https://github.com/nklayman/theia/blob/examples/add-tauri/examples/tauri/src-tauri/src/main.rs#L19
      let mut wv_clone = webview.as_mut();
      let synth_sender = core::start_synth();
      thread::spawn(move || loop {
        let (midi_sender, midi_receiver) = channel::<midi::MidiMessage>();
        match midi::create_midi_connection(midi_sender) {
          Ok(_con) => loop {
            match midi_receiver.recv().unwrap() {
              midi::MidiMessage::NoteOn { note, .. } => {
                handle_note_on(&synth_sender, &mut wv_clone, note);
              }
              midi::MidiMessage::NoteOff { note } => {
                println!("NOTE OFF midi note {} {}Hz", note, midi::midi_to_freq(note));
              }
            };
            println!("EXITED loop");
          },
          Err(_) => {
            // nothing is available to send midi messages, so just trigger some
            // frequencies regularly
            loop {
              handle_note_on(&synth_sender, &mut wv_clone, 60);
              std::thread::sleep(std::time::Duration::from_secs(1));
              handle_note_on(&synth_sender, &mut wv_clone, 64);
              std::thread::sleep(std::time::Duration::from_secs(1));
            }
          }
        };
      });
    })
    .invoke_handler(|webview, arg| {
      use cmd::Cmd::*;
      match serde_json::from_str(arg) {
        Err(e) => Err(e.to_string()),
        Ok(command) => {
          match command {
            // definitions for your custom commands from Cmd here
            StartSynth {
              argument,
              callback,
              error,
            } => tauri::execute_promise(
              webview,
              move || {
                //  your command code
                println!("{}", argument);
                Ok("Response from rust")
              },
              callback,
              error,
            ),
          }
          Ok(())
        }
      }
    })
    .build()
    .run();
}
