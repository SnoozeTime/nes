//
//
use crate::cpu::cpu::Cpu;
use crate::cpu::memory::Memory;
use crate::graphic::EmulatorInput;
use crate::joypad::{InputState, Player};
use crate::ppu::Ppu;
use crate::rom;

use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize)]
pub struct Nes {
    cpu: Cpu,
    pub ppu: Ppu,
    memory: Memory,
    rom_name: String,
    pub is_debug: bool,
    pub is_pause: bool,
    pub should_run: bool,
}

impl Nes {
    /// Empty NES console with no rom loaded.
    pub fn empty() -> Self {
        let cpu = Cpu::new();
        let ppu = Ppu::new();
        let memory = Memory::default();

        let rom_name = String::new();
        Nes {
            cpu,
            ppu,
            memory,
            rom_name,
            is_debug: false,
            is_pause: false,
            should_run: false,
        }
    }

    pub fn new(ines: rom::INesFile) -> Result<Nes, String> {
        let mut cpu = Cpu::new();
        let ppu = Ppu::new();
        let mut memory = Memory::new(&ines)?;

        // Need to set the correct PC. It is at FFFC-FFFD
        let lsb = memory.get(0xFFFC) as u16;
        let msb = memory.get(0xFFFD) as u16;
        let start_pc = (msb << 8) + lsb;
        cpu.set_pc(start_pc);

        let rom_name = String::from(ines.rom_name());
        Ok(Nes {
            cpu,
            ppu,
            memory,
            rom_name,
            is_debug: false,
            is_pause: false,
            should_run: true,
        })
    }

    pub fn width(&self) -> usize {
        256
    }

    pub fn height(&self) -> usize {
        240
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.ppu
    }

    pub fn memory(&mut self) -> &Memory {
        &self.memory
    }

    pub fn should_display(&mut self) -> bool {
        self.ppu.should_display()
    }

    pub fn get_pixel(&self, row: usize, col: usize) -> u8 {
        let idx = row * 256 + col;
        //println!("{:?}", idx);
        let pixel = self.ppu.pixels[idx];
        pixel
    }

    // Load from json file.
    pub fn load_state(path: String) -> Result<Nes, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)?;
        let n: Nes = serde_json::from_str(&json_str)?;
        Ok(n)
    }

    pub fn tick(&mut self, is_debug: bool) -> Result<u64, &'static str> {
        let cpu_cycles = self.cpu.next(&mut self.memory)?;
        self.ppu.next(3 * cpu_cycles, &mut self.memory, is_debug)?;
        Ok(cpu_cycles)
    }

    pub fn handle_event(&mut self, event: EmulatorInput) {
        match event {
            EmulatorInput::QUIT => self.should_run = false,
            EmulatorInput::PAUSE => self.is_pause = !self.is_pause,
            EmulatorInput::DEBUG => self.is_debug = !self.is_debug,
            EmulatorInput::SAVE => match self.save_state() {
                Err(err) => println!("Error while saving state: {}", err),
                Ok(_) => println!("Successfully saved to {}", self.get_save_name()),
            },
            EmulatorInput::INPUT(player, action, state) => {
                //
                match (player, state) {
                    (Player::One, InputState::Pressed) => {
                        self.memory.joypad_p1.button_down(&action)
                    }
                    (Player::Two, InputState::Pressed) => {
                        self.memory.joypad_p2.button_down(&action)
                    }
                    (Player::One, InputState::Released) => self.memory.joypad_p1.button_up(&action),
                    (Player::Two, InputState::Released) => self.memory.joypad_p2.button_up(&action),
                }
            }
        }
    }

    pub fn handle_events(&mut self, events: Vec<EmulatorInput>) {
        for event in events {
            self.handle_event(event);
        }
    }

    pub fn decompile(&mut self) {
        loop {
            self.cpu.decompile(&mut self.memory);
        }
    }

    fn get_save_name(&self) -> String {
        format!("saves/saved_{}.json", self.rom_name)
    }

    fn save_state(&self) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(self.get_save_name())
            .map_err(|err| err.to_string())?;
        let state = serde_json::to_string(&self).map_err(|err| err.to_string())?;
        write!(file, "{}", state).map_err(|err| err.to_string())?;

        Ok(())
    }
}
