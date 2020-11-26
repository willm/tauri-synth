#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use app::control::midi::{MidiMessage, MidiNote};
use app::core;
use app::{control::midi, core::SharedSynthParams};
use std::sync::mpsc::channel;
use std::thread;
use tauri::event::{emit as emit_js, listen};
use tauri::WebviewMut;

fn handle_note_on(
    synth_params: SharedSynthParams,
    webview: &mut WebviewMut,
    note_on: midi::MidiNote,
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

fn handle_note_off(
    synth_params: SharedSynthParams,
    webview: &mut WebviewMut,
    note_on: midi::MidiNote,
) {
    {
        let mut params = synth_params.lock().unwrap();
        params.note_off_producer.enqueue(1.0).unwrap();
        params.note_off_producer.enqueue(0.0).unwrap();
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
            listen("dist_amount", move |dist_amount: Option<String>| {
                let amt = dist_amount.unwrap().parse::<f32>().unwrap();
                params_clone
                    .lock()
                    .unwrap()
                    .dist_amount_producer
                    .enqueue(amt)
                    .unwrap();
            });

            let params_clone = synth_params.clone();
            listen("fm_amount", move |fm_amount: Option<String>| {
                let amt = fm_amount.unwrap().parse::<f32>().unwrap() + 0.01;
                params_clone
                    .lock()
                    .unwrap()
                    .fm_amount_producer
                    .enqueue(amt)
                    .unwrap();
            });

            let params_clone = synth_params.clone();
            listen("attack", move |attack: Option<String>| {
                let attack_time_in_seconds =
                    (f32::powf(attack.unwrap().parse::<f32>().unwrap(), 3.5) * 1.99) + 0.01;
                let attack_delta = 1.0 / (attack_time_in_seconds * 44100.0);
                params_clone
                    .lock()
                    .unwrap()
                    .attack_producer
                    .enqueue(attack_delta)
                    .unwrap();
            });

            let params_clone = synth_params.clone();
            listen("decay", move |decay: Option<String>| {
                let decay_time_in_seconds =
                    (f32::powf(1.0 - decay.unwrap().parse::<f32>().unwrap(), 3.5) * 1.99) + 0.01;
                let decay_delta = 1.0 / (decay_time_in_seconds * 44100.0);
                params_clone
                    .lock()
                    .unwrap()
                    .decay_producer
                    .enqueue(decay_delta)
                    .unwrap();
            });

            let params_clone = synth_params.clone();
            listen("sustain", move |sustain: Option<String>| {
                params_clone
                    .lock()
                    .unwrap()
                    .sustain_producer
                    .enqueue(sustain.unwrap().parse::<f32>().unwrap())
                    .unwrap();
            });

            let params_clone = synth_params.clone();
            listen("release", move |release: Option<String>| {
                let release_time_in_seconds =
                    (f32::powf(release.unwrap().parse::<f32>().unwrap(), 3.5) * 1.99) + 0.01;
                let release_delta = 1.0 / (release_time_in_seconds * 44100.0);
                params_clone
                    .lock()
                    .unwrap()
                    .release_producer
                    .enqueue(release_delta)
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
                                MidiMessage::NoteOff(note_off) => {
                                    handle_note_off(params_clone.clone(), &mut wv_clone, note_off);
                                    println!(
                                        "NOTE OFF midi note {} {}Hz",
                                        note_off.note,
                                        midi::midi_to_freq(note_off.note)
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
                        let mut trigger_note = move |note: u8, length_ms: u64| {
                            handle_note_on(
                                params_clone.clone(),
                                &mut wv_clone,
                                MidiNote {
                                    note: note,
                                    velocity: 1,
                                },
                            );
                            std::thread::sleep(std::time::Duration::from_millis(length_ms));
                        };
                        loop {
                            trigger_note(32, 200);
                            trigger_note(33, 200);
                            trigger_note(35, 200);
                            trigger_note(37, 200);
                            trigger_note(43, 100);
                            trigger_note(44, 100);
                        }
                    }
                };
            });
        })
        .build()
        .run();
}
