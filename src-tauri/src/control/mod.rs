pub mod midi;
use std::sync::mpsc;

pub fn alternate_tones(
  synth_sender: std::sync::mpsc::Sender<[f32; 3]>,
) -> Result<(), Box<dyn std::error::Error>> {
  loop {
    synth_sender.send([300.0, 303.0, 299.0])?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    synth_sender.send([600.0, 603.0, 599.0])?;
    std::thread::sleep(std::time::Duration::from_secs(1));
  }
}

pub fn read_midi_input(
  synth_sender: std::sync::mpsc::Sender<[f32; 3]>,
) -> Result<(), Box<dyn std::error::Error>> {
  let (tx, rx) = mpsc::channel::<midi::MidiMessage>();
  let _connection = midi::create_midi_connection(tx)?;

  loop {
    match rx.recv()? {
      midi::MidiMessage::NoteOn { note, .. } => {
        let freq = midi::midi_to_freq(note);
        println!("NOTE ON midi note {} {}Hz", note, freq);
        synth_sender.send([freq, freq + 3.0, freq - 1.0])?;
      }
      midi::MidiMessage::NoteOff { note } => {
        println!("NOTE OFF midi note {} {}Hz", note, midi::midi_to_freq(note));
        //synth_sender.send([0.0, 0.0, 0.0])?;
      }
    };
  }
}
