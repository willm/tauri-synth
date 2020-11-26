#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use app::control::midi;
use app::control::midi::{MidiMessage, NoteOn};
use app::core;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use tauri::event::emit as emit_js;
use tauri::WebviewMut;

fn handle_note_on(
    synth_sender: &mut rume::InputStreamProducer,
    webview: &mut WebviewMut,
    note_on: midi::NoteOn,
) {
    let freq = midi::midi_to_freq(note_on.note);
    synth_sender.enqueue(freq).unwrap();
    emit_js(webview, String::from("message"), Some(note_on)).unwrap();
}

fn main() {
    tauri::AppBuilder::new()
        .setup(move |webview, _source| {
            // https://github.com/nklayman/theia/blob/examples/add-tauri/examples/tauri/src-tauri/src/main.rs#L19
            let mut wv_clone = webview.as_mut();
            let mut synth_sender = core::start_synth();
            thread::spawn(move || loop {
                let (midi_sender, midi_receiver) = channel::<midi::MidiMessage>();
                match midi::create_midi_connection(midi_sender) {
                    Ok(_midi_connection) => {
                        emit_js(&mut wv_clone, String::from("ready"), Some(true)).unwrap();
                        loop {
                            match midi_receiver.recv().unwrap() {
                                MidiMessage::NoteOn(note_on) => {
                                    handle_note_on(&mut synth_sender, &mut wv_clone, note_on);
                                }
                                MidiMessage::NoteOff { note } => {
                                    println!(
                                        "NOTE OFF midi note {} {}Hz",
                                        note,
                                        midi::midi_to_freq(note)
                                    );
                                }
                            };
                            println!("EXITED loop");
                        }
                    }
                    Err(_) => {
                        // nothing is available to send midi messages, so just trigger some
                        // frequencies regularly
                        emit_js(&mut wv_clone, String::from("ready"), Some(true)).unwrap();
                        loop {
                            handle_note_on(
                                &mut synth_sender,
                                &mut wv_clone,
                                NoteOn {
                                    note: 60,
                                    velocity: 1,
                                },
                            );
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            handle_note_on(
                                &mut synth_sender,
                                &mut wv_clone,
                                NoteOn {
                                    note: 64,
                                    velocity: 1,
                                },
                            );
                            std::thread::sleep(std::time::Duration::from_secs(1));
                        }
                    }
                };
            });
        })
        .build()
        .run();
}
