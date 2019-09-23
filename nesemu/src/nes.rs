//
//
use crate::cpu::cpu::Cpu;
use crate::cpu::memory::Memory;
use crate::graphic::{EmulatorInput, GraphicHandler};
use crate::ppu::Ppu;
use crate::rom;
use std::time::{Duration, Instant};

use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize)]
pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    memory: Memory,
    rom_name: String,
}

impl Nes {
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
        })
    }

    // Load from json file.
    pub fn load_state(path: String) -> Result<Nes, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)?;
        let n: Nes = serde_json::from_str(&json_str)?;
        Ok(n)
    }

    // main loop
    pub fn run(&mut self, ui: &mut dyn GraphicHandler) -> Result<(), &'static str> {
        let mut is_pause = false;
        let mut is_debug = false;

        // Fixed time stamp for input polling.
        let fixed_time_stamp = Duration::new(0, 16666667);
        let mut previous_clock = Instant::now();
        let mut accumulator = Duration::new(0, 0);

        'should_run: loop {
            // Update CPU and PPU (and later APU)
            // if !is_pause {
            let cpu_cycles = self.cpu.next(&mut self.memory)?;
            self.ppu.next(3 * cpu_cycles, &mut self.memory, is_debug)?;

            // handle events.
            while accumulator > fixed_time_stamp {
                accumulator -= fixed_time_stamp;
                match ui.handle_events(&mut self.memory, is_pause) {
                    Some(EmulatorInput::QUIT) => break 'should_run,
                    Some(EmulatorInput::PAUSE) => is_pause = !is_pause,
                    Some(EmulatorInput::DEBUG) => is_debug = !is_debug,
                    Some(EmulatorInput::SAVE) => match self.save_state() {
                        Err(err) => println!("Error while saving state: {}", err),
                        Ok(_) => println!("Successfully saved to {}", self.get_save_name()),
                    },
                    None => {}
                }
                // render
                ui.display(&mut self.memory, &mut self.ppu);
            }

            accumulator += Instant::now() - previous_clock;
            previous_clock = Instant::now();

            // If pause, let's wait a bit to avoid taking all the CPU
            // if is_pause {
            //     self.ui.draw_debug(&self.memory);
            //     let ten_millis = std::time::Duration::from_millis(10);
            //     std::thread::sleep(ten_millis);
            // }
        }

        Ok(())
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
