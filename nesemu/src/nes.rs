//
//
use crate::cpu::cpu::Cpu;
use crate::cpu::memory::Memory;
use crate::graphic::{Canvas, Color, EmulatorInput};
use crate::joypad::{InputState, Player};
use crate::ppu::palette;
use crate::ppu::Ppu;
use crate::rom;

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize)]
pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    memory: Memory,
    rom_name: String,
    pub is_debug: bool,
    pub is_pause: bool,
    pub should_run: bool,
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
            is_debug: false,
            is_pause: false,
            should_run: true,
        })
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

    pub fn display(
        &mut self,
        canvas: &mut dyn Canvas,
        colors: &HashMap<u8, Color>,
        zoom_level: i32,
    ) {
        if self.should_display() {
            let bg_color = palette::get_bg_color(&self.memory.ppu_mem.palettes, colors);
            canvas.set_color(bg_color);
            canvas.clear_state();
            self.fill_canvas(canvas, zoom_level);
            canvas.show();
        }
    }

    pub fn fill_canvas(&self, canvas: &mut dyn Canvas, zoom_level: i32) {
        for row in 0..240i32 {
            for col in 0..256i32 {
                let idx = row * 256 + col;
                let pixel = self.ppu.pixels[idx as usize];

                canvas.set_color(Color::rgb(pixel.0, pixel.1, pixel.2));

                let xpixel = col * zoom_level;
                let ypixel = row * zoom_level;
                canvas.draw_rect(xpixel, ypixel, zoom_level as u32, zoom_level as u32);
            }
        }
    }

    // Load from json file.
    pub fn load_state(path: String) -> Result<Nes, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)?;
        let n: Nes = serde_json::from_str(&json_str)?;
        Ok(n)
    }

    pub fn tick(&mut self, is_debug: bool) -> Result<(), &'static str> {
        let cpu_cycles = self.cpu.next(&mut self.memory)?;
        self.ppu.next(3 * cpu_cycles, &mut self.memory, is_debug)?;
        Ok(())
    }

    pub fn handle_events(&mut self, events: Vec<EmulatorInput>) {
        for event in events {
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
                        (Player::One, InputState::Released) => {
                            self.memory.joypad_p1.button_up(&action)
                        }
                        (Player::Two, InputState::Released) => {
                            self.memory.joypad_p2.button_up(&action)
                        }
                    }
                }
            }
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
