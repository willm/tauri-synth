pub struct ADSREnveloppe {
    sample_rate: f32,
    tick: f32,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    note_off_received_at_tick: Option<f32>,
}

impl ADSREnveloppe {
    pub fn new(sample_rate: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        ADSREnveloppe {
            sample_rate,
            tick: 0.0,
            attack,
            decay,
            sustain,
            release,
            note_off_received_at_tick: None,
        }
    }

    pub fn note_off(&mut self) {
        self.note_off_received_at_tick = Some(self.tick);
    }

    pub fn next_sample(&mut self, signal: f32) -> f32 {
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
            let m = ((1.0 - self.sustain) * -1.) / (self.decay * self.sample_rate);
            // shift the x axis by the attack time
            let x = self.tick - attack_time;
            velocity = m * x + 1.0;
        }

        if let Some(note_off_tick) = self.note_off_received_at_tick {
            let m = (self.sustain * -1.) / (self.release * self.sample_rate);
            let x = self.tick - note_off_tick;
            velocity = m * x + self.sustain;
        }

        self.tick += 1.0;
        signal * velocity
    }
}

#[cfg(test)]
mod tests {
  use super::*;
  use plotlib::page::Page;
  use plotlib::repr::Plot;
  use plotlib::style::{LineStyle, LineJoin};
  use plotlib::view::ContinuousView;


  fn next_sample(env: &mut ADSREnveloppe, samples: &mut Vec<f64>) -> f32 {
      let contant_signal = 1.0_f32;
      let sample = &env.next_sample(contant_signal);
      assert!(1. >= *sample && (*sample >= 0.));
      samples.push(*sample as f64);
      *sample
  }

  fn round_to_2(x: f32) -> f32 {
      (x * 100.).round() / 100.
  }

  #[test]
  fn test_envelope() {
      let mut data: Vec<f64> = vec![];
      let sample_rate = 44_100;
      let mut env = ADSREnveloppe::new(44_100.0, 1.0, 1.0, 0.5, 1.0);
      // start
      let sample = next_sample(&mut env, &mut data);
      assert_eq!(round_to_2(sample), 0.0_f32);

      // attack
      println!("Attack");
      for _ in 1..(sample_rate - 1) {
          next_sample(&mut env, &mut data);
      }
      let sample = next_sample(&mut env, &mut data);
      assert_eq!(round_to_2(sample), 1.0_f32);

      // decay / sustain
      println!("Decay + Sustain");
      for _ in 1..(sample_rate - 1) {
          let sample = next_sample(&mut env, &mut data);
          assert!(sample > env.sustain);
      }
      let sample = next_sample(&mut env, &mut data);
      assert_eq!(round_to_2(sample), 0.5_f32);

      // release
      println!("Release");
      env.note_off();
      for _ in 1..(sample_rate - 1) {
          next_sample(&mut env, &mut data);
      }
      let sample = next_sample(&mut env, &mut data);
      assert_eq!(round_to_2(sample), 0_f32);


      let plottable_data = data.iter().enumerate().map(|x: (usize, &f64)| -> (f64, f64) {(x.0 as f64, *x.1)}).collect();
      let l1 = Plot::new(plottable_data).line_style(
          LineStyle::new()
              .colour("burlywood")
              .linejoin(LineJoin::Round),
      );
      Page::single(&ContinuousView::new().add(l1)).save("./plots/enveloppe.svg").unwrap();
  }

  #[test]
  fn test_envelope_2() {
      let mut data: Vec<f64> = vec![];
      let sample_rate = 44_100;
      let mut env = ADSREnveloppe::new(44_100.0, 1.0, 1.0, 0.2, 1.0);
      // start
      let sample = next_sample(&mut env, &mut data);
      assert_eq!(round_to_2(sample), 0.0_f32);

      // attack
      println!("Attack");
      for _ in 1..(sample_rate - 1) {
          next_sample(&mut env, &mut data);
      }
      let sample = next_sample(&mut env, &mut data);
      assert_eq!(round_to_2(sample), 1.0_f32);

      // decay / sustain
      println!("Decay + Sustain");
      for _ in 1..(sample_rate - 1) {
          let sample = next_sample(&mut env, &mut data);
          assert!(sample > env.sustain);
      }
      let sample = next_sample(&mut env, &mut data);
      assert_eq!(round_to_2(sample), 0.2_f32);

      // release
      println!("Release");
      env.note_off();
      for _ in 1..(sample_rate - 1) {
          next_sample(&mut env, &mut data);
      }
      let sample = next_sample(&mut env, &mut data);
      assert_eq!(round_to_2(sample), 0_f32);


      let plottable_data = data.iter().enumerate().map(|x: (usize, &f64)| -> (f64, f64) {(x.0 as f64, *x.1)}).collect();
      let l1 = Plot::new(plottable_data).line_style(
          LineStyle::new()
              .colour("burlywood")
              .linejoin(LineJoin::Round),
      );
      Page::single(&ContinuousView::new().add(l1)).save("./plots/enveloppe2.svg").unwrap();
  }
}