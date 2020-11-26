extern crate cpal;
pub mod enveloppe;
pub mod oscillators;
use self::enveloppe::ADSREnveloppe;
use self::oscillators::sin;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rume::Processor;
use std::sync::mpsc;

fn build_graph() -> (
    rume::SignalChain,
    rume::InputStreamProducer,
    rume::OutputStreamConsumer,
) {
    let (frequency_producer, frequency_consumer) = rume::input!(FREQUENCY_ENDPOINT);
    let (audio_out_producer, audio_out_consumer) = rume::output!(AUDIO_OUT_ENDPOINT);

    let beep = rume::graph! {
        endpoints: {
            freq: rume::InputEndpoint::new(frequency_consumer),
            audio_out: rume::OutputEndpoint::new(audio_out_producer),
        },
        processors: {
            amp: rume::Value::new(0.1),
            sine: rume::Sine::default(),
        },
        connections: {
            freq.output  ->  sine.input.0,
            amp.output   ->  sine.input.1,
            sine.output  ->  audio_out.input,
        }
    };

    (beep, frequency_producer, audio_out_consumer)
}

pub fn start_synth() -> rume::InputStreamProducer {
    let (graph, freq_producer, audio_consumer) = build_graph();

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

    freq_producer
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
