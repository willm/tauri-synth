use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{stdin, stdout, Write};

pub fn midi_to_freq(midi_note: u8) -> f32 {
  let exp = (f32::from(midi_note) + 36.376_316_562_295_91) / 12.0;
  2f32.powf(exp)
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct MidiNote {
  pub note: u8,
  pub velocity: u8,
}

#[derive(Copy, Clone)]
pub enum MidiMessage {
  NoteOn(MidiNote),
  NoteOff(MidiNote),
}

pub fn create_midi_connection(
  tx: std::sync::mpsc::Sender<MidiMessage>,
) -> Result<MidiInputConnection<()>, Box<dyn Error>> {
  // Get an input port (read from console if multiple are available)
  let mut midi_in = MidiInput::new("midir reading input")?;
  midi_in.ignore(Ignore::None);
  let in_ports = midi_in.ports();
  let in_port: &MidiInputPort = match in_ports.len() {
    0 => return Err("no input port found".into()),
    1 => {
      println!(
        "Choosing the only available input port: {}",
        midi_in.port_name(&in_ports[0])?
      );
      &in_ports[0]
    }
    _ => {
      println!("\nAvailable input ports:");
      for (i, p) in in_ports.iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p)?);
      }
      print!("Please select input port: ");
      stdout().flush().unwrap();
      let mut input = String::new();
      stdin().read_line(&mut input)?;
      let index = input.trim().parse::<usize>()?;
      in_ports.get(index).unwrap()
    }
  };
  let in_port_name = midi_in.port_name(&in_port)?;
  println!("Opening connection to port '{}'", in_port_name);
  match midi_in.connect(
    &in_port,
    "midir-read-input",
    move |_stamp, message, _| {
      if message.len() >= 3 {
        let note_on = 144;
        let note = message[1];
        let velocity = message[2];
        if message[0] == note_on {
          if velocity > 0 {
            println!("about to send note on");
            tx.send(MidiMessage::NoteOn(MidiNote { note, velocity }))
              .unwrap();
          } else {
            tx.send(MidiMessage::NoteOff(MidiNote { note, velocity }))
              .unwrap();
          }
        }
      }
    },
    (),
  ) {
    Ok(connection) => Ok(connection),
    Err(err) => Err(format!("Connection error: {}", err.description()).into()),
  }
}
