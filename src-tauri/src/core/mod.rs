pub mod dsp;
pub mod enveloppe;
pub mod oscillators;
pub use dsp::*;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rume::*;

pub fn start_synth() -> SharedSynthParams {
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
    consumer.dequeue().unwrap() * 0.1
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
