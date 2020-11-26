use rume::*;

use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Delay {
  pub input: (DelaySampleInput, DelayTimeInput),
  pub output: DelaySampleOutput,
  sample: f32,
  delay_ticks: f32,
  sample_rate: u32,
  memory: [f32; 44_100],
  read_idx: f32,
  write_idx: usize,
  buffer_size: usize,
}

impl Default for Delay {
  fn default() -> Delay {
    Delay {
      input: (DelaySampleInput, DelayTimeInput),
      output: DelaySampleOutput,
      sample: 0.0,
      delay_ticks: 0.0,
      sample_rate: 44_100,
      memory: [0.0; 44_100],
      read_idx: 0.0,
      write_idx: 0,
      buffer_size: 0,
    }
  }
}

#[rume::processor_input(Delay, DelayTimeInput)]
fn set(proc: &mut Delay, time_ms: f32) {
  proc.delay_ticks = (time_ms * 0.001) * proc.sample_rate as f32;
  proc.buffer_size = proc.memory.len();
}

#[rume::processor_input(Delay, DelaySampleInput)]
fn set(proc: &mut Delay, sample: f32) {
  proc.sample = sample;
}

#[rume::processor_output(Delay, DelaySampleOutput)]
fn get(proc: &mut Delay) -> f32 {
  proc.sample
}

#[inline(always)]
fn lerp(a: f32, b: f32, w: f32) -> f32 {
  a + w * (b - a)
}

impl Processor for Delay {
  fn prepare(&mut self, data: AudioConfig) {
    self.sample_rate = data.sample_rate;
  }

  fn process(&mut self) {
    let buffer_size = self.memory.len();

    self.memory[self.write_idx] = self.sample;
    self.write_idx = (self.write_idx + 1) % buffer_size;
    self.read_idx = (self.write_idx as f32 - self.delay_ticks) % buffer_size as f32;

    let read_idx_0 = self.read_idx as usize;
    let read_idx_1 = (read_idx_0 + 1) % buffer_size;

    let wet = lerp(
      self.memory[read_idx_0],
      self.memory[read_idx_1],
      self.read_idx % 1.0,
    );

    self.sample = lerp(self.sample, wet, 0.3);
  }
}

#[rume::processor]
pub struct Distortion {
  #[rume::processor_sample]
  sample: f32,

  #[rume::processor_input]
  amount: f32,
}

impl Processor for Distortion {
  fn prepare(&mut self, _: AudioConfig) {}

  #[inline(always)]
  fn process(&mut self) {
    self.sample = (self.amount * self.sample).tanh();
  }
}

#[rume::processor]
pub struct Sine {
  #[rume::processor_input]
  frequency: f32,

  #[rume::processor_input]
  amplitude: f32,

  #[rume::processor_input]
  amount: f32,

  #[rume::processor_output]
  sample: f32,

  phase: [f32; 2],
  inv_sample_rate: f32,
}

impl Processor for Sine {
  fn prepare(&mut self, data: AudioConfig) {
    self.inv_sample_rate = 1.0 / data.sample_rate as f32;
  }

  fn process(&mut self) {
    const TWO_PI: f32 = 2.0_f32 * std::f32::consts::PI;

    let increment = TWO_PI * self.frequency * self.inv_sample_rate;
    self.phase[0] = (self.phase[0] + increment) % TWO_PI;
    self.sample = self.phase[0].sin();

    let increment = TWO_PI * self.frequency * self.inv_sample_rate * self.sample * self.amount;
    self.phase[1] = (self.phase[1] + increment) % TWO_PI;
    self.sample += self.phase[1].sin();

    self.sample *= self.amplitude;
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

    // println!("{:?} -> {:?},{:?},{:?},{:?}", self.amplitude, self.attack_delta, self.decay_delta, self.sustain_level, self.release_delta);

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
        // panic!("Sustain {}", self.amplitude);
      }
      EnvelopeState::Release => {
        self.amplitude -= self.release_delta;
        // panic!("Release {}", self.amplitude);
        if self.amplitude <= 0.0 {
          self.amplitude = 0.0;
          self.state = EnvelopeState::Off;
        }
      }
      EnvelopeState::Off => {
        self.amplitude = 0.0;
      }
    }
    // if self.amplitude != 0.0 {
    //     println!("{}", self.amplitude);
    // }
  }
}

#[derive(Debug, Clone)]
enum FilterMode {
  LowPass = 0,
  HighPass,
  BandPass,
  Notch,
}

impl Default for FilterMode {
  fn default() -> FilterMode {
    FilterMode::LowPass
  }
}

#[derive(Debug, Default, Clone)]
struct StateVariableFilter {
  pub input: (
    StateVariableFilterSampleInput,
    StateVariableCutoffInput,
    StateVariableResonanceInput,
  ),
  pub output: StateVariableFilterSampleOutput,

  sample: f32,
  sample_rate: f32,
  damping: f32,
  frequency: f32,
  coefficients: [f32; 5],
  drive: f32,
  cutoff_in_hertz: f32,
  resonance: f32,
  mode: FilterMode,
}

#[rume::processor_input(StateVariableFilter, StateVariableFilterSampleInput)]
fn set(proc: &mut StateVariableFilter, sample: f32) {
  proc.sample = sample;
}

#[rume::processor_input(StateVariableFilter, StateVariableCutoffInput)]
fn set(proc: &mut StateVariableFilter, freq_hz: f32) {
  proc.cutoff_in_hertz = freq_hz;
}

#[rume::processor_input(StateVariableFilter, StateVariableResonanceInput)]
fn set(proc: &mut StateVariableFilter, resonance: f32) {
  proc.resonance = resonance;
}

#[rume::processor_output(StateVariableFilter, StateVariableFilterSampleOutput)]
fn get(proc: &mut StateVariableFilter) -> f32 {
  proc.sample
}

impl Processor for StateVariableFilter {
  fn prepare(&mut self, data: AudioConfig) {
    self.sample_rate = data.sample_rate as f32;
    self.mode = FilterMode::LowPass;
    self.drive = 1.0;
  }

  fn process(&mut self) {
    self.frequency = 2.0
      * std::f32::consts::PI
      * 0.25_f32
        .min(self.cutoff_in_hertz / (self.sample_rate * 2.0))
        .sin();
    self.damping = 2.0
      * (1.0 - f32::powf(self.resonance, 0.25))
        .min(2.0_f32.min(2.0 / self.frequency - self.frequency * 0.5));

    self.coefficients[3] = self.sample - self.damping * self.coefficients[2];
    self.coefficients[0] = self.coefficients[0] + self.frequency * self.coefficients[2];
    self.coefficients[1] = self.coefficients[3] - self.coefficients[0];
    self.coefficients[2] = self.frequency * self.coefficients[1] + self.coefficients[2]
      - self.drive * self.coefficients[2] * self.coefficients[2] * self.coefficients[2];

    let output = 0.5 * self.coefficients[0]; // This should be the mode!

    self.coefficients[3] = self.sample - self.damping * self.coefficients[2];
    self.coefficients[0] = self.coefficients[0] + self.frequency * self.coefficients[2];
    self.coefficients[1] = self.coefficients[3] - self.coefficients[0];
    self.coefficients[2] = self.frequency * self.coefficients[1] + self.coefficients[2]
      - self.drive * self.coefficients[2] * self.coefficients[2] * self.coefficients[2];

    self.sample = output + 0.5 * self.coefficients[0];
  }
}

pub struct SynthParams {
  pub freq_producer: rume::InputStreamProducer,
  pub fm_amount_producer: rume::InputStreamProducer,
  pub dist_amount_producer: rume::InputStreamProducer,
  pub note_on_producer: rume::InputStreamProducer,
  pub note_off_producer: rume::InputStreamProducer,
  pub attack_producer: rume::InputStreamProducer,
  pub decay_producer: rume::InputStreamProducer,
  pub sustain_producer: rume::InputStreamProducer,
  pub release_producer: rume::InputStreamProducer,
}

pub type SharedSynthParams = Arc<Mutex<SynthParams>>;

pub fn build_graph() -> (
  rume::SignalChain,
  SharedSynthParams,
  rume::OutputStreamConsumer,
) {
  let (freq_producer, freq_consumer) = rume::input!(FREQUENCY_ENDPOINT);
  let (fm_amount_producer, fm_amount_consumer) = rume::input!(FM_AMOUNT_ENDPOINT);
  let (dist_amount_producer, dist_amount_consumer) = rume::input!(DIST_AMOUNT_ENDPOINT);
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
          fm_amt: rume::InputEndpoint::new(fm_amount_consumer),
          dist_amt: rume::InputEndpoint::new(dist_amount_consumer),
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
          sine: Sine::default(),
          dist: Distortion::default(),
          dly: Delay::default(),
          val: Value::new(125.0),
      },
      connections: {
          freq.output    ->  sine.input.0,
          env.output     ->  sine.input.1,
          fm_amt.output  ->  sine.input.2,

          attack.output   ->  env.input.0,
          decay.output    ->  env.input.1,
          sustain.output  ->  env.input.2,
          release.output  ->  env.input.3,

          note_on.output  ->  env.input.4,
          note_off.output ->  env.input.5,

          sine.output     ->  dist.input.0,
          dist_amt.output ->  dist.input.1,
          dist.output     ->  dly.input.0,
          val.output      ->  dly.input.1,
          dly.output      ->  audio_out.input,
      }
  };

  (
    beep,
    Arc::new(Mutex::new(SynthParams {
      freq_producer,
      fm_amount_producer,
      dist_amount_producer,
      note_on_producer,
      note_off_producer,
      attack_producer,
      decay_producer,
      sustain_producer,
      release_producer,
    })),
    audio_out_consumer,
  )
}
