use super::cpu::memory::Memory;
use super::ppu::Ppu;

/// Uses SDL 2 to render graphics.

extern crate sdl2;
use self::sdl2::{Sdl, VideoSubsystem, EventPump};
use self::sdl2::video::Window;
use self::sdl2::pixels::Color;
use self::sdl2::event::Event;
use self::sdl2::rect::Rect;
use self::sdl2::keyboard::Keycode;
use std::time::Duration;
use self::sdl2::render::{RenderTarget, Canvas, WindowCanvas};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;

pub struct Graphics {
    zoom_level: u32,
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    canvas: WindowCanvas,
    event_pump: EventPump,
}


impl Graphics {

    pub fn new(zoom_level: u32) -> Result<Graphics, String> {
        let sdl_context = sdl2::init()
            .map_err(|err| err.to_string())?;
        let video_subsystem = sdl_context.video()
            .map_err(|err| err.to_string())?;

        let window = video_subsystem
            .window("NES emulator", WIDTH*zoom_level, HEIGHT*zoom_level)
            .position_centered()
            .opengl()
            .build()
            .map_err(|err| err.to_string())?;

        let mut canvas = window
            .into_canvas()
            .software()
            .build()
            .map_err(|err| err.to_string())?;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let event_pump = sdl_context.event_pump().map_err(|err| err.to_string())?;

        Ok(Graphics { 
            zoom_level,
            sdl_context,
            video_subsystem,
            canvas,
            event_pump
        })
    }

    // This is called in the main loop and will listen for 
    // input pressed. If a key is pressed, it will modify
    // the register accordingly.
    pub fn handle_events(&mut self, _mem: &mut Memory) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit{..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), ..}
                => {
                    return true;
                },
                _ => {},

            }
        }

        false
    }

    // Will get the buffer from the ppu and display it on screen.
    // PPU decides whether the pixel should be displayed.
    pub fn display(&mut self, memory: &Memory, ppu: &mut Ppu) {

        if ppu.should_display() {
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
    }
}

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


