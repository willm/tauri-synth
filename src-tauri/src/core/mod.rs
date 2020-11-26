extern crate cpal;
pub mod enveloppe;
pub mod oscillators;
use self::enveloppe::ADSREnveloppe;
use self::oscillators::sin;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rume::*;
use std::sync::mpsc;

#[derive(Debug,Clone)]
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
struct Envelope {
    #[rume::processor_output]
    amplitude:f32,

    sample_rate:f32,
    state:EnvelopeState,

    #[rume::processor_input]
    attack_delta:f32,
    #[rume::processor_input]
    decay_delta:f32,
    #[rume::processor_input]
    sustain_level:f32,
    #[rume::processor_input]
    release_delta:f32,

    #[rume::processor_input]
    note_on:f32,
    #[rume::processor_input]
    note_off:f32,
}

impl Processor for Envelope {
    fn prepare(&mut self, data:rume::AudioConfig) {
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

        self.attack_delta = 0.0001;
        self.decay_delta = 0.0001;
        self.sustain_level = 0.0;
        self.release_delta = 0.01;

        match self.state {
            EnvelopeState::Attack => {
                self.amplitude = self.amplitude + self.attack_delta;
                if self.amplitude >= 1.0 {
                    self.amplitude = 1.0;
                    self.state = EnvelopeState::Decay;
                }
            },
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
            },
            EnvelopeState::Sustain => {
                self.amplitude = self.sustain_level;
            },
            EnvelopeState::Release => {
                self.amplitude -= self.release_delta;
                if self.amplitude <= 0.0 {
                    self.amplitude = 0.0;
                    self.state = EnvelopeState::Off;
                }
            },
            EnvelopeState::Off => {
                self.amplitude = 0.0;
            }
        }
    }
}

fn build_graph() -> (
    rume::SignalChain,
    SynthParams,
    rume::OutputStreamConsumer,
) {
    let (freq_producer, freq_consumer) = rume::input!(FREQUENCY_ENDPOINT);
    let (note_on_producer, note_on_consumer) = rume::input!(NOTE_ON_ENDPOINT);
    let (sustain_producer, sustain_consumer) = rume::input!(SUSTAIN_ENDPOINT);
    let (audio_out_producer, audio_out_consumer) = rume::output!(AUDIO_OUT_ENDPOINT);

    let beep = rume::graph! {
        endpoints: {
            freq: rume::InputEndpoint::new(freq_consumer),
            note_on: rume::InputEndpoint::new(note_on_consumer),
        sustain: rume::InputEndpoint::new(sustain_consumer),
            audio_out: rume::OutputEndpoint::new(audio_out_producer),
        },
        processors: {
            amp: rume::Value::new(0.1),
            env: Envelope::default(),
            sine: rume::Sine::default(),
        },
        connections: {
            freq.output    ->  sine.input.0,
            note_on.output ->  env.input.4,
            sustain.output ->  env.input.2,
            env.output     ->  sine.input.1,
            sine.output    ->  audio_out.input,
        }
    };

    (beep, SynthParams{freq_producer, note_on_producer, sustain_producer}, audio_out_consumer)
}

pub struct SynthParams {
    pub freq_producer: rume::InputStreamProducer,
    pub note_on_producer: rume::InputStreamProducer,
    pub sustain_producer: rume::InputStreamProducer,
}

pub fn start_synth() -> SynthParams {
    let (graph, synth_params, audio_consumer) = build_graph();

    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();

        match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), graph, audio_consumer),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), graph, audio_consumer),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), graph, audio_consumer),
        }
    });

    synth_params
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut graph: rume::SignalChain,
    mut consumer: rume::OutputStreamConsumer,
) where
    T: cpal::Sample,
{
    let channels = config.channels as usize;

    graph.prepare(config.sample_rate.0.into());

    let mut next_value = move || {
        graph.render(1);
        consumer.dequeue().unwrap()
    };

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            |err| eprintln!("an error occurred on stream: {}", err),
        )
        .unwrap();
    stream.play().unwrap();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
