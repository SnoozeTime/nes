#![allow(unused)]
use crate::cpu::memory::Memory;
use serde_derive::{Deserialize, Serialize};
use std::default::Default;
use tracing::{debug, info, trace};
mod filters;
use filters::FilterChain;
pub mod memory;

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
 *1 1111 (1F) => 30
1 1101 (1D) => 28
1 1011 (1B) => 26
1 1001 (19) => 24
1 0111 (17) => 22
1 0101 (15) => 20
1 0011 (13) => 18
1 0001 (11) => 16

Notes with base length 12 (4/4 at 75 bpm):
1 1110 (1E) => 32  (96 times 1/3, quarter note triplet)
1 1100 (1C) => 16  (48 times 1/3, eighth note triplet)
1 1010 (1A) => 72  (48 times 1 1/2, dotted quarter)
1 1000 (18) => 192 (Whole note)
1 0110 (16) => 96  (Half note)
1 0100 (14) => 48  (Quarter note)
1 0010 (12) => 24  (Eighth note)
1 0000 (10) => 12  (Sixteenth)
 *
 * */
const LENGTH_COUNTER_LOOKUP: [u8; 0x20] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ApuMemory {
    /// 0x4015
    /// status: Determine whether the channels are silenced or not
    pub status_reg: u8,

    /// 0x4000 to 0x4003 (included)
    /// Pulse 1 - This is a square wave.
    /// --------------------------------------------
    pub pulse_1_reg1: u8,
    pub pulse_1_reg2: u8,
    pulse_1: Pulse,

    pulse_2: Pulse,

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
                self.pulse_1.timer = self.pulse_1.timer & 0b11100000000 | (value as u16);
                info!(timer = %self.pulse_1.timer);
            }
            0x4003 => {
                self.pulse_1.timer = (value as u16 & 0b111) << 8 | self.pulse_1.timer & 0b11111111;
                self.pulse_1.reset();

                // Counter to 0. When 0, channel is silenced.
                // 0x4003 = LLLL.L.xxx
                self.pulse_1.length_counter.value = LENGTH_COUNTER_LOOKUP[(value >> 3) as usize];
                debug!(
                    "Will set pulse 1 length counter to {}",
                    self.pulse_1.length_counter.value
                );
                info!(timer = %self.pulse_1.timer);
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
                self.pulse_2.timer = self.pulse_2.timer & 0b11100000000 | (value as u16);
            }
            0x4007 => {
                self.pulse_2.timer = (value as u16 & 0b111) << 8 | self.pulse_2.timer & 0b11111111;
                self.pulse_2.reset();

                // Counter to 0. When 0, channel is silenced.
                // 0x4003 = LLLL.L.xxx
                self.pulse_2.length_counter.value = LENGTH_COUNTER_LOOKUP[(value >> 3) as usize];
            }

            0x4015 => {
                self.status_reg = value;
                self.pulse_1.enabled = value & 0b1 == 0b1;
                self.pulse_2.enabled = value & 0b10 == 0b10;
            }

            0x4017 => {
                let mode = value & 0b1000_0000;
                self.frame_counter.mode = mode; // won't be 1 but it's ok, the condition is on 0.
                if mode > 0 {
                    self.tick_envelopes();
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

    fn tick_envelopes(&mut self) {
        self.pulse_1.envelope.tick();
        self.pulse_2.envelope.tick();
    }

    fn tick_length_counters(&mut self) {
        self.pulse_1.length_counter.tick();
        self.pulse_2.length_counter.tick();
    }

    pub fn is_pulse1_enabled(&self) -> bool {
        self.status_reg & 0b1 == 0b1
    }

    pub fn is_pulse2_enabled(&self) -> bool {
        self.status_reg & 0b10 == 0b10
    }

    pub fn is_triangle_enabled(&self) -> bool {
        self.status_reg & 0b100 == 0b100
    }

    pub fn is_noise_enabled(&self) -> bool {
        self.status_reg & 0b1000 == 0b1000
    }

    pub fn is_dmc_enabled(&self) -> bool {
        self.status_reg & 0b10000 == 0b10000
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
    cycles: u64,

    // Rate at which we take a sample
    sample_timer: u64,
    sample_timer_rate: u64,
    samples: Vec<i16>,
    extra: u64,

    #[serde(skip)]
    filters: FilterChain,
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
    seq_index: u8,

    /// Timer value. Number of clock before with clock the sequencer.
    timer: u16,
    current_timer: u16,
    length_counter_load: u8,
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
        if self.timer == 0 {
            return;
        }

        if self.current_timer == 0 {
            // clock the sequencer :D
            //self.seq_index = (self.seq_index.wrapping_sub(1)) % 7;
            if self.seq_index == 7 {
                self.seq_index = 0;
            } else {
                self.seq_index += 1;
            }
            debug!("seq index -> {:?}", self.seq_index);
            self.current_timer = self.timer;
        } else {
            self.current_timer -= 1;
        }
    }

    /// After writing to 4003/4007
    fn reset(&mut self) {
        self.seq_index = 0;
        // TODO reset envelope
        //self.current_timer = self.timer;
    }

    pub fn sample(&self) -> f64 {
        // volume * duty * length counter...
        let duty = DUTY_VALUES[self.duty_cycle as usize][self.seq_index as usize];

        let value = if self.length_counter.value != 0 {
            self.envelope.volume() as f64 * duty as f64
        } else {
            0.0
        };

        trace!(msg = "take sample", value = %value, duty_cycle = %self.duty_cycle, seq_index = %self.seq_index, duty = %duty, timer = %self.current_timer);
        value * 100.0
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
        }
    }

    pub fn next(&mut self, cpu_ticks: u64, mem: &mut Memory) {
        self.cycles += cpu_ticks;

        for _ in 0..cpu_ticks {
            // Frame counter timer.
            mem.apu_mem.frame_counter.tick();

            // Clock everything.
            if self.cycles & 1 == 0 {
                // clock pulse
                mem.apu_mem.pulse_1.tick();
                mem.apu_mem.pulse_2.tick();
            }

            // Length counter and envelopes update.
            if mem.apu_mem.frame_counter.is_1st_quarter() {
                mem.apu_mem.tick_envelopes();
            } else if mem.apu_mem.frame_counter.is_half() {
                mem.apu_mem.tick_envelopes();
                mem.apu_mem.pulse_1.length_counter.tick();
                mem.apu_mem.pulse_2.length_counter.tick();
            } else if mem.apu_mem.frame_counter.is_3rd_quarter() {
                mem.apu_mem.tick_envelopes();
            } else if mem.apu_mem.frame_counter.is_last() {
                mem.apu_mem.tick_envelopes();
                mem.apu_mem.pulse_1.length_counter.tick();
                mem.apu_mem.pulse_2.length_counter.tick();
            }

            // Instead of taking a lot of samples (Frequency of APU is > 1 Mhz). let's just sample at
            // 44100Hz.
            // Should we take a sample?
            if self.sample_timer == 0 {
                // take a sample and reset timer.
                self.sample_timer = self.sample_timer_rate + self.extra;
                self.extra = (self.extra + 1) % 2;

                let pulse_1_sample = if mem.apu_mem.pulse_1.enabled {
                    mem.apu_mem.pulse_1.sample()
                } else {
                    0.0
                };

                let pulse_2_sample = if mem.apu_mem.pulse_2.enabled {
                    mem.apu_mem.pulse_2.sample()
                } else {
                    0.0
                };

                let mut mixed = pulse_1_sample + pulse_2_sample;
                debug!(msg = "sample", sample = %mixed);
                //mixed = self.filters.tick(mixed);

                self.samples.push(mixed as i16);
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
