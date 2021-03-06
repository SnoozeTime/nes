#![allow(unused)]
use crate::cpu::memory::Memory;
use serde_derive::{Deserialize, Serialize};
use std::default::Default;
use tracing::{debug, info, trace};
mod filters;
use filters::FilterChain;

// Same as CPU (one frame is 60Hz)
const TICK_PER_FRAME: f64 = 29780.0;
// Computer audio is 44100Hz. 60 frames per second. 44100/60
const SAMPLES_PER_FRAME: f64 = 735.0;
const SAMPLE_TIMER_RATE: f64 = TICK_PER_FRAME / SAMPLES_PER_FRAME;
const FRAME_COUNTER_RATE: f64 = TICK_PER_FRAME / 4.0;

const DUTY_VALUES: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

/*
 * Look up table for the length counter value. The index is written to $4015
 * */
const LENGTH_COUNTER_LOOKUP: [u8; 0x20] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const TRIANGLE_WAVE: [f64; 32] = [
    15.0, 14.0, 13.0, 12.0, 11.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0, 0.0, 1.0,
    2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
];

#[derive(Debug)]
pub struct ApuLevels {
    pulse_1: f64,
    pulse_2: f64,
    triangle: f64,
    master: f64,
}

impl Default for ApuLevels {
    fn default() -> Self {
        Self {
            pulse_1: 1.0,
            pulse_2: 1.0,
            triangle: 1.0,
            master: 10_000.0,
        }
    }
}

impl ApuLevels {
    pub fn set_pulse1_level(&mut self, pulse_1: f64) {
        self.pulse_1 = pulse_1.min(1.0);
    }
    pub fn set_pulse2_level(&mut self, pulse_2: f64) {
        self.pulse_2 = pulse_2.min(1.0);
    }
    pub fn set_triangle_level(&mut self, triangle: f64) {
        self.triangle = triangle.min(1.0);
    }

    pub fn set_master_level(&mut self, master: f64) {
        self.master = master.min(10_000.0);
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ApuMemory {
    /// 0x4000 to 0x4003 (included)
    /// Pulse 1 - This is a square wave.
    /// --------------------------------------------
    pub pulse_1_reg1: u8,
    pub pulse_1_reg2: u8,
    pulse_1: Pulse,
    pulse_2: Pulse,
    triangle: Triangle,

    frame_counter: FrameCounter,
    /// True if something has changed since last write/read
    pub dirty: bool,
}

impl ApuMemory {
    pub fn write(&mut self, addr: usize, value: u8) {
        self.dirty = true;
        match addr {
            // PULSE 1
            // ---------------------------------
            0x4000 => {
                self.pulse_1_reg1 = value;
                self.pulse_1.duty_cycle = value >> 6;
                self.pulse_1.envelope.period = value & 0b1111;
                self.pulse_1.envelope.do_loop = value & 0b00100000 == 0b00100000;
                self.pulse_1.envelope.enabled = value & 0b00010000 == 0;
                self.pulse_1.length_counter.halt_flag_set = value & 0b00100000 == 0b00100000;
                info!(
                    "got 0x4000 {:08b} => {}",
                    value, self.pulse_1.length_counter.halt_flag_set
                );
                info!(duty = %self.pulse_1.duty_cycle);
            }
            0x4001 => self.pulse_1_reg2 = value,

            // Timer for the first pulse channel. Set via 0x4002 and 0x4003
            // HHH.LLLL.LLLL
            // 0x4002 = LLLL.LLLL
            // 0x4003 = xxxx.xHHH
            0x4002 => {
                self.pulse_1.timer.set_low(value);
                //self.pulse_1.timer = self.pulse_1.timer & 0b11100000000 | (value as u16);
                info!(timer = ?self.pulse_1.timer);
            }
            0x4003 => {
                //self.pulse_1.timer = (value as u16 & 0b111) << 8 | self.pulse_1.timer & 0b11111111;
                self.pulse_1.timer.set_high(value);
                self.pulse_1.reset();

                // Counter to 0. When 0, channel is silenced.
                // 0x4003 = LLLL.L.xxx
                if self.pulse_1.enabled {
                    self.pulse_1.length_counter.value =
                        LENGTH_COUNTER_LOOKUP[(value >> 3) as usize];
                }

                debug!(
                    "Will set pulse 1 length counter to {}",
                    self.pulse_1.length_counter.value
                );
                info!(timer = ?self.pulse_1.timer);
            }

            // PULSE 2
            // ---------------------------------------
            0x4004 => {
                self.pulse_2.duty_cycle = value >> 6;
                self.pulse_2.envelope.period = value & 0b1111;
                self.pulse_2.envelope.do_loop = value & 0b00100000 == 0b00100000;
                self.pulse_2.envelope.enabled = value & 0b00010000 == 0;
                self.pulse_2.length_counter.halt_flag_set = value & 0b00100000 == 0b00100000;
            }
            0x4005 => self.pulse_1_reg2 = value,

            // Timer for the first pulse channel. Set via 0x4002 and 0x4003
            // HHH.LLLL.LLLL
            // 0x4002 = LLLL.LLLL
            // 0x4003 = xxxx.xHHH
            0x4006 => {
                //self.pulse_2.timer = self.pulse_2.timer & 0b11100000000 | (value as u16);
                self.pulse_2.timer.set_low(value);
            }
            0x4007 => {
                self.pulse_2.timer.set_high(value);
                //self.pulse_2.timer = (value as u16 & 0b111) << 8 | self.pulse_2.timer & 0b11111111;
                self.pulse_2.reset();

                // Counter to 0. When 0, channel is silenced.
                // 0x4003 = LLLL.L.xxx
                if self.pulse_2.enabled {
                    self.pulse_2.length_counter.value =
                        LENGTH_COUNTER_LOOKUP[(value >> 3) as usize];
                }
            }

            // TRIANGLE
            // -----------------------------------------------
            0x4008 => {
                info!("0x4008 triangle => {:08b}", value);
                let length_counter_halt = value & 0b1000_0000 == 0b1000_0000;
                self.triangle.length_counter.halt_flag_set = length_counter_halt;
                self.triangle.linear_counter.control_flag = length_counter_halt;
                self.triangle.linear_counter.reload_value = value & 0b0111_1111;
            }
            0x400A => {
                info!("0x400A triangle => {:08b}", value);
                self.triangle.timer.set_low(value);
            }
            0x400B => {
                info!("0x400B triangle => {:08b}", value);
                self.triangle.timer.set_high(value);
                self.triangle.length_counter.value = LENGTH_COUNTER_LOOKUP[(value >> 3) as usize];
                self.triangle.linear_counter.reload_flag = true;
            }

            // ----------------------------------------------------
            0x4015 => {
                self.pulse_1.set_enabled(value & 0b1 == 0b1);
                self.pulse_2.set_enabled(value & 0b10 == 0b10);
                self.triangle.set_enabled(value & 0b100 == 0b100);
            }

            0x4017 => {
                let mode = value & 0b1000_0000;
                self.frame_counter.mode = mode; // won't be 1 but it's ok, the condition is on 0.
                if mode > 0 {
                    self.tick_envelopes_and_linear_counter();
                    self.tick_length_counters();
                }
            }
            _ => (),
        }
    }

    pub fn read(&mut self) -> u8 {
        let mut res = 0;
        if self.pulse_1.length_counter.value > 0 {
            res |= 0b1;
        }
        if self.pulse_2.length_counter.value > 0 {
            res |= 0b10;
        }
        res
    }

    fn tick_envelopes_and_linear_counter(&mut self) {
        self.pulse_1.envelope.tick();
        self.pulse_2.envelope.tick();
        self.triangle.linear_counter.tick();
    }

    fn tick_length_counters(&mut self) {
        self.pulse_1.length_counter.tick();
        self.pulse_2.length_counter.tick();
        self.triangle.length_counter.tick();
    }
}

// -----------------------------------------
//
#[derive(Debug, Serialize, Deserialize, Default)]
struct FrameCounter {
    mode: u8,
    current_count: u64,
}

impl FrameCounter {
    // one cpu cycle
    pub fn tick(&mut self) {
        self.current_count = self.current_count + 1;
        if self.mode == 0 && self.current_count > 29829 {
            self.current_count = 0;
        } else if self.current_count > 37281 {
            self.current_count = 0;
        }
    }

    pub fn reset(&mut self) {
        self.current_count = 0;
    }

    /// Should clock envelopes and triangle's linear counter
    pub fn is_1st_quarter(&self) -> bool {
        self.current_count == 7457
    }

    /// Should clock envelopes and triangle's linear counter
    /// and also length counter, sweep units
    pub fn is_half(&self) -> bool {
        self.current_count == 14913
    }

    /// Should clock envelopes and triangle's linear counter
    pub fn is_3rd_quarter(&self) -> bool {
        self.current_count == 22371
    }

    /// Should clock envelopes and triangle's linear counter
    /// and also length counter, sweep units
    pub fn is_last(&self) -> bool {
        match (self.mode, self.current_count) {
            (0, 29829) => true,
            (1, 37281) => true,
            _ => false,
        }
    }

    /// TODO Implement that
    pub fn is_interrupt(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------
//
#[derive(Default, Serialize, Deserialize, Debug)]
struct LengthCounter {
    value: u8,
    halt_flag_set: bool,
}

impl LengthCounter {
    fn tick(&mut self) {
        if !self.halt_flag_set && self.value > 0 {
            self.value -= 1;
        }
        debug!(msg = "Length counter", flag = %self.halt_flag_set,
               value = %self.value);
    }
}

// ----------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Apu {
    /// Keep track how many cycles since the beginning.
    pub cycles: u64,

    // Rate at which we take a sample
    sample_timer: u64,
    sample_timer_rate: u64,
    samples: Vec<i16>,
    extra: u64,

    #[serde(skip)]
    filters: FilterChain,

    #[serde(skip)]
    pub levels: ApuLevels,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Envelope {
    /// if true, the volume will decrease every tick.
    enabled: bool,
    envelop: bool,
    do_loop: bool,
    counter: u8,

    // period and constant volume are same value.
    period: u8,
    timer: u8,
}

impl Envelope {
    fn volume(&self) -> u8 {
        if self.envelop && self.enabled {
            self.counter
        } else if self.envelop && !self.enabled {
            0
        } else {
            self.period
        }
    }

    fn tick(&mut self) {
        if self.timer == 0 {
            if self.counter == 0 {
                self.counter = 15;
            } else {
                self.counter -= 1
            }
            self.timer = self.period;
        } else {
            self.timer -= 1;
        }
    }
}

// --------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default)]
struct Triangle {
    enabled: bool,
    length_counter: LengthCounter,
    seq_index: usize,
    linear_counter: LinearCounter,

    /// Timer value. Number of clock before with clock the sequencer.
    timer: Timer,
}

impl Triangle {
    /// tick every CPU clock.
    pub fn tick(&mut self) {
        if self.timer.tick() {
            // clock the sequencer :D
            //self.seq_index = (self.seq_index.wrapping_sub(1)) % 7;
            if self.seq_index == 31 {
                self.seq_index = 0;
            } else {
                self.seq_index += 1;
            }
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter.value = 0;
        }
    }
    /// Get a sample from the triangle wave.
    /// Will be silenced if:
    /// - disabled
    /// - length counter or linear counter set to 0
    /// Volume is defined by the sequncer.
    pub fn sample(&self) -> f64 {
        info!(msg = "Triangle_sample", enabled = %self.enabled, length_counter = %self.length_counter.value, linear_counter = %self.linear_counter.current_value, seq_index = %self.seq_index);
        if !self.enabled {
            return 0.0;
        }

        if self.length_counter.value == 0 {
            return 0.0;
        }

        if self.linear_counter.current_value == 0 {
            return 0.0;
        }

        TRIANGLE_WAVE[self.seq_index]
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct LinearCounter {
    control_flag: bool,
    current_value: u8,
    reload_value: u8,
    reload_flag: bool,
}

impl LinearCounter {
    fn tick(&mut self) {
        if self.reload_flag {
            self.current_value = self.reload_value;
        } else if self.current_value > 0 {
            self.current_value -= 1;
        }

        if !self.control_flag {
            self.reload_flag = false;
        }
    }
}
// -------------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Default)]
struct Timer {
    timer: u16,
    current_timer: u16,
}

impl Timer {
    /// Return true if should tick the next step.
    fn tick(&mut self) -> bool {
        if self.timer == 0 {
            return false;
        }
        if self.current_timer == 0 {
            // clock the sequencer :D
            self.current_timer = self.timer;
            return true;
        } else {
            self.current_timer -= 1;
        }

        false
    }

    fn set_low(&mut self, low: u8) {
        self.timer = self.timer & 0b11100000000 | (low as u16);
    }

    fn set_high(&mut self, high: u8) {
        self.timer = (high as u16 & 0b111) << 8 | self.timer & 0b11111111;
    }
}

// --------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default)]
struct Pulse {
    enabled: bool,

    envelope: Envelope,
    length_counter: LengthCounter,
    /// Set by the duty
    /// 00 -> 01000000
    /// 01 -> 01100000
    /// 10 -> 01111000
    /// 11 -> 10011111
    duty_cycle: u8,

    /// Paired with the duty cycle, will determine if high or low
    /// output.
    seq_index: usize,

    /// Timer value. Number of clock before with clock the sequencer.
    timer: Timer,
    // timer: u16,
    // current_timer: u16,
}

impl Pulse {
    pub fn new() -> Self {
        Self {
            seq_index: 0,
            ..Self::default()
        }
    }

    /// Should be done every second CPU tick.
    pub fn tick(&mut self) {
        if self.timer.tick() {
            if self.seq_index == 7 {
                self.seq_index = 0;
            } else {
                self.seq_index += 1;
            }
        }

        //        if self.timer == 0 {
        //            return;
        //        }
        //
        //        if self.current_timer == 0 {
        //            // clock the sequencer :D
        //            //self.seq_index = (self.seq_index.wrapping_sub(1)) % 7;
        //            if self.seq_index == 7 {
        //                self.seq_index = 0;
        //            } else {
        //                self.seq_index += 1;
        //            }
        //            self.current_timer = self.timer;
        //        } else {
        //            self.current_timer -= 1;
        //        }
    }

    /// After writing to 4003/4007
    fn reset(&mut self) {
        self.seq_index = 0;
        // TODO reset envelope
        //self.current_timer = self.timer;
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter.value = 0;
        }
    }

    pub fn sample(&self) -> f64 {
        if self.length_counter.value == 0 {
            return 0.0;
        }
        // volume * duty * length counter...
        let duty = DUTY_VALUES[self.duty_cycle as usize][self.seq_index];

        self.envelope.volume() as f64 * duty as f64
    }
}

impl Apu {
    pub fn new() -> Self {
        // shoganai
        let sample_timer_rate = 40; //SAMPLE_TIMER_RATE.round() as u64;
        let sample_timer = sample_timer_rate;
        let samples = Vec::with_capacity(1024);
        Self {
            cycles: 0,
            sample_timer,
            sample_timer_rate,
            samples,
            extra: 0,
            filters: FilterChain::default(),
            levels: ApuLevels::default(),
        }
    }

    pub fn next(&mut self, cpu_ticks: u64, mem: &mut Memory) {
        //self.cycles += cpu_ticks;

        for _ in 0..cpu_ticks {
            self.cycles += 1;
            // Clock everything.
            if self.cycles & 1 == 0 {
                // clock pulse
                mem.apu_mem.pulse_1.tick();
                mem.apu_mem.pulse_2.tick();

                // Frame counter timer.
                mem.apu_mem.frame_counter.tick();
            }
            mem.apu_mem.triangle.tick();

            // Length counter and envelopes update.
            if mem.apu_mem.frame_counter.is_1st_quarter() {
                mem.apu_mem.tick_envelopes_and_linear_counter();
            } else if mem.apu_mem.frame_counter.is_half() {
                mem.apu_mem.tick_envelopes_and_linear_counter();
                mem.apu_mem.tick_length_counters();
            } else if mem.apu_mem.frame_counter.is_3rd_quarter() {
                mem.apu_mem.tick_envelopes_and_linear_counter();
            } else if mem.apu_mem.frame_counter.is_last() {
                mem.apu_mem.tick_envelopes_and_linear_counter();
                mem.apu_mem.tick_length_counters();
            }

            // Instead of taking a lot of samples (Frequency of APU is > 1 Mhz). let's just sample at
            // 44100Hz.
            // Should we take a sample?
            if self.sample_timer == 0 {
                // take a sample and reset timer.
                self.sample_timer = self.sample_timer_rate + self.extra;
                self.extra = (self.extra + 1) % 2;

                let pulse_1_sample = self.levels.pulse_1 * mem.apu_mem.pulse_1.sample();
                let pulse_2_sample = self.levels.pulse_2 * mem.apu_mem.pulse_2.sample();
                let triangle_sample = self.levels.triangle * mem.apu_mem.triangle.sample();

                // at first linear approximation
                // pulse_out = 0.00752 * (pulse1 + pulse2)
                // tnd_out = 0.00851 * triangle + 0.00494 * noise + 0.00335 * dmc
                let mut mixed =
                    0.00752 * (pulse_1_sample + pulse_2_sample) + 0.00851 * triangle_sample;
                debug!(msg = "sample", sample = %mixed);
                mixed = self.filters.tick(mixed);

                self.samples.push((self.levels.master * mixed) as i16);
            }
            self.sample_timer -= 1;
        }
    }

    /// Will drain all our samples to send to the audio queue.
    /// TODO allocate every frame. Is that ok? maybe easier to pass a
    /// buffer to the function
    pub fn samples(&mut self) -> Vec<i16> {
        self.samples.drain(..).collect()
    }
}
