extern crate cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use std::sync::mpsc;

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

struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

struct Enveloppe {
    sample_rate: f32,
    tick: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    note_off_received_at_tick: Option<f32>,
}

impl Enveloppe {
    fn new(sample_rate: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Enveloppe {
            sample_rate,
            tick: 0.0,
            attack,
            decay,
            sustain,
            release,
            note_off_received_at_tick: None,
        }
    }

    fn note_off(&mut self) {
        self.note_off_received_at_tick = Some(self.tick);
    }

    fn next_sample(&mut self, signal: f32) -> f32 {
        // the equation for a straight line is y = mx + b;
        // m is the slope of the curve and is calculated as (change in y) / (change in x)

        let mut velocity = 1.0_f32;

        let attack_time = self.attack * self.sample_rate;
        if self.tick < attack_time {
            let m = self.attack / self.sample_rate;
            velocity = m * self.tick + 0.0;
        }

        let decay_time = attack_time + (self.decay * self.sample_rate);
        if self.tick >= attack_time && self.tick < decay_time {
            let m = (self.sustain) / decay_time;
            velocity = m * self.tick + 0.0;
        }

        if let Some(note_off_tick) = self.note_off_received_at_tick {
            let m = (note_off_tick + self.release) / self.sample_rate;
            velocity = m * self.tick + 0.0;
        }

        self.tick += 1.0;
        let value = signal * velocity;
        println!("{}", value);
        value
    }
}

#[test]
fn test_envelope() {
    let contant_signal = 1.0_f32;
    let mut data: Vec<(f64, f64)> = vec![];
    let mut env = Enveloppe::new(44_100.0, 1.0, 1.0, 0.5, 1.0);
    // start
    let mut sample = env.next_sample(contant_signal);
    let mut i = 0.0_f64;
    data.push((i, sample as f64));
    assert_eq!(sample.round(), 0.0_f32);

    // attack
    println!("Attack");
    for _ in 1..43_999 {
        i += 1_f64;
        sample = env.next_sample(contant_signal);
        data.push((i, sample as f64));
    }
    i += 1_f64;
    let sample = env.next_sample(contant_signal);
    data.push((i, sample as f64));
    assert_eq!(sample.round(), 1.0_f32);

    // decay / sustain
    println!("Decay + Sustain");
    for _ in 1..43_999 {
        env.next_sample(contant_signal);
    }
    let sample = env.next_sample(contant_signal);
    assert_eq!(((sample * 10.0_f32).round() / 10_f32), 0.5_f32);

    // release
    println!("Release");
    env.note_off();
    for _ in 1..43_999 {
        env.next_sample(contant_signal);
    }
    let sample = env.next_sample(contant_signal);
    let plot = Plot::new(data);
    // The 'view' describes what set of data is drawn
    let v = ContinuousView::new()
        .add(plot)
        .x_range(0., 9_000_000.)
        .y_range(-1., 1.)
        .x_label("Some varying variable")
        .y_label("The response of something");

    // A page with a single view is then saved to an SVG file
    Page::single(&v).save("scatter.svg").unwrap();
    //assert_eq!(((sample * 10.0_f32).round() / 10_f32), 0.0_f32);
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
            velocity += 0.000001;
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
