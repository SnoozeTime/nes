pub mod memory;
use super::cpu::memory::Memory;
use self::memory::RegisterType;

extern crate sdl2;
use self::sdl2::{Sdl, VideoSubsystem};
use self::sdl2::video::Window;
use self::sdl2::pixels::Color;
use self::sdl2::event::Event;
use self::sdl2::rect::Rect;
use self::sdl2::keyboard::Keycode;
use std::time::Duration;
use self::sdl2::render::{RenderTarget, Canvas, WindowCanvas};

struct Tile {
    plane1: [u8; 8],
    plane2: [u8; 8],
}

impl Tile {
    fn new(pattern_table: &[u8], sprite_nb: usize) -> Tile {

        let mut plane1 = [0;8];
        let mut plane2 = [0;8];

        for i in 0..8 {
            plane1[i] = pattern_table[16*sprite_nb + i];
            plane2[i] = pattern_table[16*sprite_nb + i + 8];
        }

        Tile { plane1, plane2 }
    }

    fn draw<T: RenderTarget>(&self, canvas: &mut Canvas<T>, x: i32, y: i32) {

        for yline in 0..8 {
            let v1 = self.plane1[yline];
            let v2 = self.plane2[yline];
            for xline in 0..8 {
                let bit1 = (v1 >> 8-(xline+1)) & 1;
                let bit2 = ((v2 >> 8-(xline+1)) & 1) << 1;
                let v = bit1 + bit2;
                    if v == 1 {
                        canvas.set_draw_color(Color::RGB(255, 0, 0));
                    } else if v == 2 {
                        canvas.set_draw_color(Color::RGB(0, 255, 0));
                    } else if v == 3 {
                        canvas.set_draw_color(Color::RGB(0, 0, 255));
                    } else {
                        canvas.set_draw_color(Color::RGB(0, 0, 0));
                    }
                    // // A draw a rectangle which almost fills our window with it !
                    canvas.fill_rect(Rect::new(x + xline as i32, y + yline as i32, 1, 1)).unwrap();
            }
        }
    }

}


/*
 * Fun times.
 * PPU has internal memory (pattern tables, nametable, attributes and so on)
 * It communicates with CPU through registers. Registers are in the CPU
 * memory (from $2000 to $2007) -> https://wiki.nesdev.com/w/index.php/PPU_registers
 *
 */

pub struct Ppu {

    // 262 line per frame.
    line: usize,
    // 341 cycle per line.
    cycle: usize,

    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    canvas: WindowCanvas,
}

impl Ppu {

    pub fn new() -> Ppu {
        let line = 0;
        let cycle = 0;
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("NES emulator", 256, 240)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().software().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Ppu { line, cycle, sdl_context, video_subsystem, canvas}
    }

    /// Execute one PPU cycle
    /// There are 3 ppu cycles for each cpu cycle.
    // PPU cycles are a bit more complicated than CPU
    // https://wiki.nesdev.com/w/index.php/PPU_frame_timing
    // https://wiki.nesdev.com/w/index.php/PPU_rendering
    //
    // In this emulator, I chose to run the CPU first, then the PPU. The CPU
    // will return the number of cycles it had executed and the PPU will execute
    // 3 times as many cycles.
    pub fn next(&mut self, cycles_to_exec: u8, memory: &mut Memory) -> Result<(), &'static str> {

        let ppu_mask = memory.ppu_mem.peek(RegisterType::PPUMASK);
        let ppu_status = memory.ppu_mem.peek(RegisterType::PPUSTATUS);

        // no rendering. just add the cycles.
        // No way we add more than one line at a time in the current code...
        for _ in 0..cycles_to_exec {
            if self.line < 241 {
                if (ppu_mask & 0x2 == 0x2) || (ppu_mask & 0x8 == 0x8) {
                    //println!("Show background Line {} ccycle {} ", self.line, self.cycle);
                }
            } else {
                // inside VBlank :)
            }

            self.cycle = (self.cycle + 1) % 341;
            if self.cycle == 0 {
                self.line += 1;
            }


            if self.line == 241 && self.cycle == 1 {
                memory.ppu_mem.update(RegisterType::PPUSTATUS, ppu_status | 0x80);
            }

            if self.line == 261 && self.cycle == 0 {
                memory.ppu_mem.update(RegisterType::PPUSTATUS, ppu_status & !0x80);
                let nametable = &memory.ppu_mem.ppu_mem[0x2000..0x2400];

                self.canvas.clear();
                let mut index: usize = 0;
                for row in 0..30i32 {
                    for col in 0..32i32 {
                        index = 32*(row as usize) + (col as usize);
                        let tile = Tile::new(&memory.ppu_mem.ppu_mem[0x1000..0x2000], nametable[index] as usize);
                        tile.draw(&mut self.canvas, col*8, row*8);
                    }
                }
                self.canvas.present();
            }

            self.line = self.line % 262;
        }

        Ok(())
    }
}



