use rume::*;

use std::sync::{Arc, Mutex};

#[rume::processor]
pub struct Sine {
    #[rume::processor_input]
    frequency: f32,

    #[rume::processor_input]
    amplitude: f32,

    #[rume::processor_output]
    sample: f32,

    phase: f32,
    sample_rate: u32,
}

impl Processor for Sine {
    fn prepare(&mut self, data: AudioConfig) {
        self.sample_rate = data.sample_rate;
    }

    fn process(&mut self) {
        const TWO_PI: f32 = 2.0_f32 * std::f32::consts::PI;
        let increment = TWO_PI * self.frequency * (1.0_f32 / self.sample_rate as f32);
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

        // println!("{:?} -> {:?},{:?},{:?},{:?}", self.amplitude, self.attack_delta, self.decay_delta, self.sustain_level, self.release_delta);

        match self.state {
            EnvelopeState::Attack => {
                self.amplitude += self.attack_delta;
                if self.amplitude >= 1.0 {
                    self.amplitude = 1.0;
                    println!("Decay");
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

pub struct SynthParams {
    pub freq_producer: rume::InputStreamProducer,
    pub note_on_producer: rume::InputStreamProducer,
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
        attack_producer: rume::InputStreamProducer,
        decay_producer: rume::InputStreamProducer,
        sustain_producer: rume::InputStreamProducer,
        release_producer: rume::InputStreamProducer,
    ) -> SharedSynthParams {
        Arc::new(Mutex::new(Self {
            freq_producer,
            note_on_producer,
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
    let (attack_producer, attack_consumer) = rume::input!(ATTACK_ENDPOINT);
    let (decay_producer, decay_consumer) = rume::input!(DECAY_ENDPOINT);
    let (sustain_producer, sustain_consumer) = rume::input!(SUSTAIN_ENDPOINT);
    let (release_producer, release_consumer) = rume::input!(RELEASE_ENDPOINT);
    let (audio_out_producer, audio_out_consumer) = rume::output!(AUDIO_OUT_ENDPOINT);

    let beep = rume::graph! {
        endpoints: {
            freq: rume::InputEndpoint::new(freq_consumer),
            note_on: rume::InputEndpoint::new(note_on_consumer),
            attack: rume::InputEndpoint::new(attack_consumer),
            decay: rume::InputEndpoint::new(decay_consumer),
            sustain: rume::InputEndpoint::new(sustain_consumer),
            release: rume::InputEndpoint::new(release_consumer),
            audio_out: rume::OutputEndpoint::new(audio_out_producer),
        },
        processors: {
            env: Envelope::default(),
            sine: Sine::default(),
        },
        connections: {
            freq.output    ->  sine.input.0,
            note_on.output ->  env.input.4,
            attack.output   ->  env.input.0,
            decay.output   ->  env.input.1,
            sustain.output   ->  env.input.2,
            release.output   ->  env.input.3,
            env.output     ->  sine.input.1,
            sine.output    ->  audio_out.input,
        }
    };

    (
        beep,
        SynthParams::new(freq_producer, note_on_producer, attack_producer, decay_producer, sustain_producer, release_producer),
        audio_out_consumer,
    )
}
