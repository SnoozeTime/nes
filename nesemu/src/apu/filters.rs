//! This is quite dirty. I got the equations from blargg's comment on
//! NES forum: http://forums.nesdev.com/viewtopic.php?p=44255#p44255
//!
//! Other emulators are using different equations for some reason.
//!
//! TODO something better :') or at least without magic numbers.
use std::default::Default;
use std::f64::consts::PI;

#[derive(Debug)]
struct HighPass {
    prev_filter: f64,
    prev_sample: f64,
    k: f64,
}

impl HighPass {
    fn tick(&mut self, obs: f64) -> f64 {
        let new_filtered = self.k * self.prev_filter + obs - self.prev_sample;
        self.prev_filter = new_filtered;
        self.prev_sample = obs;
        new_filtered
    }
}

#[derive(Debug)]
struct LowPass {
    prev_filter: f64,
}

impl LowPass {
    fn tick(&mut self, obs: f64) -> f64 {
        let new_filter = (obs - self.prev_filter) * 0.815686;
        self.prev_filter = new_filter;
        new_filter
    }
}

#[derive(Debug)]
pub struct FilterChain {
    high_pass_1: HighPass,
    high_pass_2: HighPass,
    low_pass: LowPass,
}

impl Default for FilterChain {
    fn default() -> Self {
        Self {
            high_pass_1: HighPass {
                prev_filter: 0.0,
                prev_sample: 0.0,
                k: 0.996039,
            },
            high_pass_2: HighPass {
                prev_filter: 0.0,
                prev_sample: 0.0,
                k: 0.999835,
            },
            low_pass: LowPass { prev_filter: 0.0 },
        }
    }
}

impl FilterChain {
    pub fn tick(&mut self, mut obs: f64) -> f64 {
        obs = self.high_pass_1.tick(obs);
        obs = self.high_pass_2.tick(obs);
        self.low_pass.tick(obs)
    }
}
