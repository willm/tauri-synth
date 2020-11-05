extern crate cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
pub mod enveloppe;

const PI: f32 = std::f32::consts::PI;
pub fn start_synth() -> mpsc::Sender<[f32; 3]> {
    let (synth_sender, synth_receiver) = mpsc::channel::<[f32; 3]>();
    std::thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("failed to find a default output device");
        let config = device.default_output_config().unwrap();

        match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), synth_receiver),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), synth_receiver),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), synth_receiver),
        }
    });
    synth_sender
}

fn sin(sample_clock: f32, sample_rate: f32, freq: f32) -> f32 {
    (sample_clock * freq * 2.0 * PI / sample_rate).sin()
}


fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    rx: std::sync::mpsc::Receiver<[f32; 3]>,
) where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut freqs: [f32; 3] = [0.0, 0.0, 0.0];
    let mut velocity = 0_f32;
    let mut next_value = move || {
        freqs = match rx.try_recv() {
            // try_recv tries to get a value without blocking
            Ok(v) => {
                velocity = 0_f32;
                v
            }
            _ => freqs,
        };
        sample_clock = (sample_clock + 1.0) % sample_rate;
        let mut s = 0f32;
        //s += sin(sample_clock, sample_rate, freqs[0]);
        for &f in freqs.iter() {
            s += sin(sample_clock, sample_rate, f) * velocity;
        }
        if velocity < 1_f32 {
            velocity += 0.000_001;
        }
        s
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            err_fn,
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
