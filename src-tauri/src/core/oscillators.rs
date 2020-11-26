const PI: f32 = std::f32::consts::PI;
pub fn sin(sample_clock: f32, sample_rate: f32, freq: f32) -> f32 {
  (sample_clock * freq * 2.0 * PI / sample_rate).sin()
}
