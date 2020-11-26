use rume::*;

use std::sync::{Arc, Mutex};

#[rume::processor]
pub struct Mixer {
  #[rume::processor_input]
  channel1: f32,
  #[rume::processor_input]
  channel2: f32,
  #[rume::processor_input]
  channel3: f32,

  #[rume::processor_input]
  amplitude: f32,

  #[rume::processor_output]
  sample: f32,
}

impl Processor for Mixer {
  fn prepare(&mut self, _data: AudioConfig) {
    
  }

  fn process(&mut self) {
    self.sample = (self.channel1 +
                   self.channel2 +
                   self.channel3) * self.amplitude * (1.0/3.0);
  }
}

#[rume::processor]
pub struct Sine {
  #[rume::processor_input]
  frequency: f32,

  #[rume::processor_input]
  amplitude: f32,

  #[rume::processor_input]
  frequency_offset: f32,

  #[rume::processor_output]
  sample: f32,

  phase: f32,
  sample_rate: u32,
  inv_sample_rate: f32,
  two_pi: f32,
}

impl Processor for Sine {
  fn prepare(&mut self, data: AudioConfig) {
    self.sample_rate = data.sample_rate;
    self.inv_sample_rate = 1.0_f32 / self.sample_rate as f32;
    self.amplitude = 1.0;
    self.frequency_offset = 1.0;
    self.two_pi = 2.0_f32 * std::f32::consts::PI;
  }

  fn process(&mut self) {
    let increment = self.two_pi * self.frequency * self.frequency_offset * self.inv_sample_rate;
    self.phase = (self.phase + increment) % TWO_PI;
    self.sample = self.phase.sin() * self.amplitude;
  }
}

#[derive(Debug, Clone)]
enum EnvelopeState {
  Off = 0,
  Attack,
  Decay,
  Sustain,
  Release,
}

impl Default for EnvelopeState {
  fn default() -> EnvelopeState {
    EnvelopeState::Off
  }
}

#[rume::processor]
pub struct Envelope {
  #[rume::processor_output]
  amplitude: f32,

  sample_rate: f32,
  state: EnvelopeState,

  #[rume::processor_input]
  attack_delta: f32,
  #[rume::processor_input]
  decay_delta: f32,
  #[rume::processor_input]
  sustain_level: f32,
  #[rume::processor_input]
  release_delta: f32,

  #[rume::processor_input]
  note_on: f32,
  #[rume::processor_input]
  note_off: f32,
}

impl Processor for Envelope {
  fn prepare(&mut self, data: rume::AudioConfig) {
    self.sample_rate = data.sample_rate as f32;
  }

  fn process(&mut self) {
    if self.note_on >= 1.0 {
      self.state = EnvelopeState::Attack;
      self.note_on = 0.0;
    }

    if self.note_off >= 1.0 {
      self.state = EnvelopeState::Release;
      self.note_off = 0.0;
    }

    match self.state {
      EnvelopeState::Attack => {
        self.amplitude += self.attack_delta;
        if self.amplitude >= 1.0 {
          self.amplitude = 1.0;
          self.state = EnvelopeState::Decay;
        }
      }
      EnvelopeState::Decay => {
        self.amplitude -= self.decay_delta;
        if self.amplitude <= self.sustain_level {
          if self.amplitude <= 0.0 {
            self.state = EnvelopeState::Off;
          } else {
            self.amplitude = self.sustain_level;
            self.state = EnvelopeState::Sustain;
          }
        }
      }
      EnvelopeState::Sustain => {
        self.amplitude = self.sustain_level;
      }
      EnvelopeState::Release => {
        self.amplitude -= self.release_delta;
        if self.amplitude <= 0.0 {
          self.amplitude = 0.0;
          self.state = EnvelopeState::Off;
        }
      }
      EnvelopeState::Off => {
        self.amplitude = 0.0;
      }
    }
  }
}

pub struct SynthParams {
  pub freq_producer: rume::InputStreamProducer,
  pub note_on_producer: rume::InputStreamProducer,
  pub note_off_producer: rume::InputStreamProducer,
  pub attack_producer: rume::InputStreamProducer,
  pub decay_producer: rume::InputStreamProducer,
  pub sustain_producer: rume::InputStreamProducer,
  pub release_producer: rume::InputStreamProducer,
}

pub type SharedSynthParams = Arc<Mutex<SynthParams>>;

impl SynthParams {
  pub fn new(
    freq_producer: rume::InputStreamProducer,
    note_on_producer: rume::InputStreamProducer,
    note_off_producer: rume::InputStreamProducer,
    attack_producer: rume::InputStreamProducer,
    decay_producer: rume::InputStreamProducer,
    sustain_producer: rume::InputStreamProducer,
    release_producer: rume::InputStreamProducer,
  ) -> SharedSynthParams {
    Arc::new(Mutex::new(Self {
      freq_producer,
      note_on_producer,
      note_off_producer,
      attack_producer,
      decay_producer,
      sustain_producer,
      release_producer,
    }))
  }
}

pub fn build_graph() -> (
  rume::SignalChain,
  SharedSynthParams,
  rume::OutputStreamConsumer,
) {
  let (freq_producer, freq_consumer) = rume::input!(FREQUENCY_ENDPOINT);
  let (note_on_producer, note_on_consumer) = rume::input!(NOTE_ON_ENDPOINT);
  let (note_off_producer, note_off_consumer) = rume::input!(NOTE_OFF_ENDPOINT);
  let (attack_producer, attack_consumer) = rume::input!(ATTACK_ENDPOINT);
  let (decay_producer, decay_consumer) = rume::input!(DECAY_ENDPOINT);
  let (sustain_producer, sustain_consumer) = rume::input!(SUSTAIN_ENDPOINT);
  let (release_producer, release_consumer) = rume::input!(RELEASE_ENDPOINT);
  let (audio_out_producer, audio_out_consumer) = rume::output!(AUDIO_OUT_ENDPOINT);

  let beep = rume::graph! {
      endpoints: {
          freq: rume::InputEndpoint::new(freq_consumer),
          note_on: rume::InputEndpoint::new(note_on_consumer),
          note_off: rume::InputEndpoint::new(note_off_consumer),
          attack: rume::InputEndpoint::new(attack_consumer),
          decay: rume::InputEndpoint::new(decay_consumer),
          sustain: rume::InputEndpoint::new(sustain_consumer),
          release: rume::InputEndpoint::new(release_consumer),
          audio_out: rume::OutputEndpoint::new(audio_out_producer),
      },
      processors: {
          env: Envelope::default(),
          mixer: Mixer::default(),
          sine1: Sine::default(),
          sine2: Sine::default(),
          sine3: Sine::default(),
          sin2Offset: rume::Value::new(0.5),
          sin3Offset: rume::Value::new(0.3),
      },
      connections: {
            sin2Offset.output -> sine2.input.2,
            sin3Offset.output -> sine3.input.2,
          freq.output ->  sine1.input.0,
          freq.output ->  sine2.input.0,
          freq.output ->  sine3.input.0,
          note_on.output ->  env.input.4,
          note_off.output ->  env.input.5,
          attack.output ->  env.input.0,
          decay.output ->  env.input.1,
          sustain.output ->  env.input.2,
          release.output ->  env.input.3,
          env.output ->  sine3.input.1,
          sine1.output -> sine2.input.1,
          sine2.output -> sine3.input.1,
          sine3.output -> audio_out.input, //mixer.input.2,
          //mixer.output ->  audio_out.input,
      }
  };

  (
    beep,
    SynthParams::new(
      freq_producer,
      note_on_producer,
      note_off_producer,
      attack_producer,
      decay_producer,
      sustain_producer,
      release_producer,
    ),
    audio_out_consumer,
  )
}
