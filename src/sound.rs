use std::collections::VecDeque;

use crate::tia::TiaAudioSample;

#[derive(Clone, Copy, Debug)]
struct PartialSample {
    pub value: f32,
    pub clocks: u16
}

impl PartialSample {
    pub fn new() -> Self {
        Self { value: 0.0, clocks: 0 }
    }
}

pub struct AudioConverter {
    clocks_per_sample: u16,
    partial_samples: [PartialSample; 2],
    samples: Vec<u8>,
    chan: bool
}

impl AudioConverter {
    pub fn new(clocks_per_sample: u16) -> Self {
        Self { clocks_per_sample, partial_samples: [PartialSample::new(); 2], samples: Vec::new(), chan: false }
    }

    pub fn convert(&mut self, channels: &mut [VecDeque<TiaAudioSample>; 2]) -> &[u8]{
        if self.samples.len() & 1 != 0 {
            let last = *self.samples.last().unwrap();
            self.samples.clear();
            self.samples.push(last);
        } else {
            self.samples.clear();
        }

        while let Some(s) = self.generate(&mut channels[self.chan as usize]) {
            self.samples.push(s);
            self.chan = !self.chan;
        }

        &self.samples[0 .. self.samples.len() & !1]
    }

    pub fn generate(&mut self, source: &mut VecDeque<TiaAudioSample>) -> Option<u8> {
        if source.len() < 2 {
            return None
        }

        let mut s0 = source[0];
        let s1 = source[1];
        let clocks = s1.cycles.wrapping_sub(s0.cycles);
        let clocks_needed = self.clocks_per_sample - self.partial_samples[self.chan as usize].clocks;

        if clocks > clocks_needed {
            s0.cycles = s0.cycles.wrapping_add(clocks_needed);
            source[0] = s0;
            let s = self.partial_samples[self.chan as usize].value + s0.value as f32 * clocks_needed as f32;
            self.partial_samples[self.chan as usize] = PartialSample::new();

            let x = (s / self.clocks_per_sample as f32 + 0.5) as u8;
            Some(x)
        } else {
            self.partial_samples[self.chan as usize].value += s0.value as f32 * clocks as f32;
            self.partial_samples[self.chan as usize].clocks += clocks;
            source.pop_front();
            self.generate(source)
        }
    }
}