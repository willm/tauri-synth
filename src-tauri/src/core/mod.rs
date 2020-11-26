extern crate cpal;
pub mod enveloppe;
pub mod oscillators;
use self::enveloppe::ADSREnveloppe;
use self::oscillators::sin;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rume::Processor;
use std::sync::mpsc;

fn build_graph() -> (rume::SignalChain, rume::OutputStreamConsumer) {
    let (producer, consumer) = rume::output!(AUDIO_OUT_ENDPOINT);

    let beep = rume::graph! {
        endpoints: {
            audio_out: rume::OutputEndpoint::new(producer),
        },
        processors: {
            freq: rume::Value::new(220.0),
            amp: rume::Value::new(0.1),
            sine: rume::Sine::default(),
        },
        connections: {
            freq.output  ->  sine.input.0,
            amp.output   ->  sine.input.1,
            sine.output  ->  audio_out.input,
        }
    };
    (beep, consumer)
}

pub fn start_synth() -> mpsc::Sender<[f32; 3]> {
    let (synth_sender, synth_receiver) = mpsc::channel::<[f32; 3]>();

    let (graph, consumer) = build_graph();

    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();

        match config.sample_format() {
            cpal::SampleFormat::F32 => {
                run::<f32>(&device, &config.into(), synth_receiver, graph, consumer)
            }
            cpal::SampleFormat::I16 => {
                run::<i16>(&device, &config.into(), synth_receiver, graph, consumer)
            }
            cpal::SampleFormat::U16 => {
                run::<u16>(&device, &config.into(), synth_receiver, graph, consumer)
            }
        }
    });
    synth_sender
}

fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    rx: std::sync::mpsc::Receiver<[f32; 3]>,
    mut graph: rume::SignalChain,
    mut consumer: rume::OutputStreamConsumer,
) where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut env = ADSREnveloppe::new(sample_rate, 0.5, 0.05, 0., 0.);
    let mut sample_clock = 0f32;
    let mut freqs: [f32; 3] = [0.0, 0.0, 0.0];
    let mut next_value = move || {
        freqs = match rx.try_recv() {
            // try_recv tries to get a value without blocking
            Ok(v) => {
                env.reset();
                v
            }
            _ => freqs,
        };
        sample_clock = (sample_clock + 1.0) % sample_rate;
        let mut s = 0f32;
        //s += sin(sample_clock, sample_rate, freqs[0]);
        for &f in freqs.iter() {
            s += sin(sample_clock, sample_rate, f);
        }
        env.next_sample(s)
    };

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
