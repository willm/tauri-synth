#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use app::control::midi::{MidiMessage, NoteOn};
use app::core;
use app::{control::midi, core::SharedSynthParams};
use std::sync::mpsc::channel;
use std::thread;
use tauri::event::{emit as emit_js, listen};
use tauri::WebviewMut;

fn handle_note_on(
    synth_params: SharedSynthParams,
    webview: &mut WebviewMut,
    note_on: midi::NoteOn,
) {
    let freq = midi::midi_to_freq(note_on.note);
    {
        let mut params = synth_params.lock().unwrap();
        params.freq_producer.enqueue(freq).unwrap();
        params.note_on_producer.enqueue(1.0).unwrap();
        params.note_on_producer.enqueue(0.0).unwrap();
    }
    emit_js(webview, String::from("message"), Some(note_on)).unwrap();
}

fn main() {
    tauri::AppBuilder::new()
        .setup(move |webview, _source| {
            // https://github.com/nklayman/theia/blob/examples/add-tauri/examples/tauri/src-tauri/src/main.rs#L19
            let mut wv_clone = webview.as_mut();
            let synth_params = core::start_synth();

            let params_clone = synth_params.clone();
            listen("sustain", move |decay: Option<String>| {
                let decay_delta = 1.0 - decay.unwrap().parse::<f32>().unwrap();
                let decay_delta = (decay_delta * 2.0) / 44100.0;
                params_clone
                    .lock()
                    .unwrap()
                    .sustain_producer
                    .enqueue(decay_delta)
                    .unwrap();
            });

            let params_clone = synth_params.clone();
            thread::spawn(move || loop {
                let (midi_sender, midi_receiver) = channel::<midi::MidiMessage>();
                match midi::create_midi_connection(midi_sender) {
                    Ok(_midi_connection) => {
                        emit_js(&mut wv_clone, String::from("ready"), Some(true)).unwrap();
                        loop {
                            match midi_receiver.recv().unwrap() {
                                MidiMessage::NoteOn(note_on) => {
                                    handle_note_on(params_clone.clone(), &mut wv_clone, note_on);
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
                                params_clone.clone(),
                                &mut wv_clone,
                                NoteOn {
                                    note: 60,
                                    velocity: 1,
                                },
                            );
                            std::thread::sleep(std::time::Duration::from_secs(4));
                            handle_note_on(
                                params_clone.clone(),
                                &mut wv_clone,
                                NoteOn {
                                    note: 64,
                                    velocity: 1,
                                },
                            );
                            std::thread::sleep(std::time::Duration::from_secs(4));
                        }
                    }
                };
            });
        })
        .build()
        .run();
}
