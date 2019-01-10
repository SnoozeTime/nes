
struct Pulse {
    duty: u8,
    envelope_loop: bool,
    constant_volume: bool,
    envelope_or_volume: u8, //volume

    sweep_enabled: bool,
    period: u8,
    negate: bool,
    shift: u8,

    timer: u16,
    length_counter_load: u8,
}

impl Pulse {
    fn new() -> Pulse {
        Pulse { 
            duty: 0,
            envelope_loop: false,
            constant_volume: false,
            envelope_or_volume: 0,
            sweep_enabled: false,
            period: 9,
            negate: false,
            shift: 0,
            timer: 0,
            length_counter_load: 0,
        }
    }

    fn apply_reg1(&mut self, value: u8) {
        self.duty = (value & 0xC0) >> 6;
        self.envelope_loop = ((value & 0x20) >> 5) & 1 == 1;
        self.constant_volume = ((value & 0x10) >> 4) & 1 == 1;
        self.envelope_or_volume = value & 0xF;
    }

    fn apply_reg2(&mut self, value: u8) {
        self.sweep_enabled = (value >> 7) & 1 == 1;
        self.period = (value & 0x70) >> 4;
        self.negate = (value >> 3) & 1 == 1;
        self.shift = value & 0x7;
    }

    fn apply_reg3(&mut self, value: u8) {
        self.timer = self.timer & 0xFF00 | value as u16;
    }

    fn apply_reg4(&mut self, value: u8) {
        self.timer = (value as u16 & 0x7) << 8 | (self.timer & 0xFF);
        self.length_counter_load = (value & !0x7) >> 3;
    }

}

struct ApuMemory {
    // channels
   pulse_1: Pulse, 
   pulse_2: Pulse, 
}

impl ApuMemory {

    pub fn new() -> ApuMemory {
        ApuMemory {
            pulse_1: Pulse::new(),
            pulse_2: Pulse::new(),
        }
    }

    pub fn write(&mut self, addr: usize, value: u8) {
        
        match addr {

            // pulse 1
            0x4000 => self.pulse_1.apply_reg1(value),
            0x4001 => self.pulse_1.apply_reg2(value),
            0x4002 => self.pulse_1.apply_reg3(value),
            0x4003 => self.pulse_1.apply_reg4(value),
            
            // pulse 2
            0x4004 => self.pulse_2.apply_reg1(value),
            0x4005 => self.pulse_2.apply_reg2(value),
            0x4006 => self.pulse_2.apply_reg3(value),
            0x4007 => self.pulse_2.apply_reg4(value),
        
            _ => {},
        }

    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn write_to_pulse1() {
        let mut apu = ApuMemory::new();
        apu.write(0x4000, 0b10010011);
        assert_eq!(0b10, apu.pulse_1.duty);
        assert_eq!(false, apu.pulse_1.envelope_loop);
        assert_eq!(true, apu.pulse_1.constant_volume);
        assert_eq!(3, apu.pulse_1.envelope_or_volume);
        apu.write(0x4001, 0b11000111);
        assert_eq!(true, apu.pulse_1.sweep_enabled);
        assert_eq!(0b100, apu.pulse_1.period);
        assert_eq!(false, apu.pulse_1.negate);
        assert_eq!(7, apu.pulse_1.shift);
        apu.write(0x4002, 0b11001010);
        apu.write(0x4003, 0b11111101);
        assert_eq!(1482, apu.pulse_1.timer);
        assert_eq!(0b11111, apu.pulse_1.length_counter_load);
    }

    #[test]
    fn write_to_pulse2() {
        let mut apu = ApuMemory::new();
        apu.write(0x4004, 0b10010011);
        assert_eq!(0b10, apu.pulse_2.duty);
        assert_eq!(false, apu.pulse_2.envelope_loop);
        assert_eq!(true, apu.pulse_2.constant_volume);
        assert_eq!(3, apu.pulse_2.envelope_or_volume);
        apu.write(0x4005, 0b11000111);
        assert_eq!(true, apu.pulse_2.sweep_enabled);
        assert_eq!(0b100, apu.pulse_2.period);
        assert_eq!(false, apu.pulse_2.negate);
        assert_eq!(7, apu.pulse_2.shift);
        apu.write(0x4006, 0b11001010);
        apu.write(0x4007, 0b11111101);
        assert_eq!(1482, apu.pulse_2.timer);
        assert_eq!(0b11111, apu.pulse_2.length_counter_load);
    }


}
