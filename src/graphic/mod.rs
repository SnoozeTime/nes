use super::cpu::memory::Memory;
use super::ppu::{Ppu, TileRowInfo};
use super::ppu::palette;
use std::collections::HashMap;
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
    colors: HashMap<u8, Color>,
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
            event_pump,
            colors: palette::build_default_colors(),
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
    pub fn real_display(&mut self, memory: &Memory, ppu: &mut Ppu) {

        if ppu.should_display() {

            self.canvas.clear();
            let mut index: usize = 0;
            for row in 0..240i32 {
                let rowattr = row / 4;
                for col in 0..32i32 {
                    index = 32*(row as usize) + (col as usize);
                    let tilerow = &ppu.virtual_buffer[index];
                    let xtile = col*8*(self.zoom_level as i32);
                    let ytile = row*(self.zoom_level as i32);
                    // Now draw
                    draw(&mut self.canvas, xtile, ytile, self.zoom_level, &tilerow);
                }
            }
            self.canvas.present();
        }
    }

    // Will get the buffer from the ppu and display it on screen.
    // PPU decides whether the pixel should be displayed.
    pub fn display(&mut self, memory: &Memory, ppu: &mut Ppu) {

        if ppu.should_display() {
            let ppu_mem = &memory.ppu_mem.ppu_mem;
            let nametable = &ppu_mem[0x2000..0x2400];
            let pattern_table = &ppu_mem[0x1000..0x2000];

            self.canvas.clear();
            let mut index: usize = 0;
            for row in 0..30i32 {
                let rowattr = row / 4;
                for col in 0..32i32 {
                    index = 32*(row as usize) + (col as usize);
                    let tile = Tile::new(self.zoom_level, pattern_table, nametable[index] as usize);

                    let xtile = col*8*(self.zoom_level as i32);
                    let ytile = row*8*(self.zoom_level as i32);

                    // fetch attributes for this tile.
                    let colattr = col / 4;
                    let attr_idx = 0x3c0 + 8*rowattr+colattr;
                    let attr_byte = nametable[attr_idx as usize];

                    let box_row = (row%4) / 2;
                    let box_col = (col%4) / 2;
                    let attribute = match (box_row, box_col) {
                        (0, 0) => attr_byte & 0b11,
                        (0, 1) => (attr_byte & 0b1100) >> 2 ,
                        (1, 0) => (attr_byte & 0b110000) >> 4,
                        (1, 1) => (attr_byte & 0b11000000) >> 6,
                        _ => panic!("Not possible"),
                    };

                    let palette = palette::get_bg_palette(attribute, ppu_mem, &self.colors).unwrap();
                    // Now draw
                    tile.draw(&mut self.canvas, xtile, ytile, &palette);
                }
            }
            self.canvas.present();
        }
    }
}

struct Tile {
    plane1: [u8; 8],
    plane2: [u8; 8],
    zoom_level: u32,
}

impl Tile {
    fn new(zoom_level: u32, pattern_table: &[u8], sprite_nb: usize) -> Tile {

        let mut plane1 = [0;8];
        let mut plane2 = [0;8];

        if sprite_nb != 0x24 {
            println!("SPRITE NB {}", sprite_nb);
        }
        for i in 0..8 {
            plane1[i] = pattern_table[16*sprite_nb + i];
            plane2[i] = pattern_table[16*sprite_nb + i + 8];
        }

        Tile { plane1, plane2, zoom_level}
    }

    fn draw<T: RenderTarget>(&self, canvas: &mut Canvas<T>, x: i32, y: i32, palette: &palette::Palette) {

        for yline in 0..8 {
            let v1 = self.plane1[yline];
            let v2 = self.plane2[yline];
            for xline in 0..8 {
                let bit1 = (v1 >> 8-(xline+1)) & 1;
                let bit2 = ((v2 >> 8-(xline+1)) & 1) << 1;
                let v = bit1 + bit2;
                if v == 1 {
                    canvas.set_draw_color(palette.color1);
                } else if v == 2 {
                    canvas.set_draw_color(palette.color2);
                } else if v == 3 {
                    canvas.set_draw_color(palette.color3);
                } else {
                    canvas.set_draw_color(palette.background);
                }

                let zoom = self.zoom_level as i32;
                let xpixel = x + (xline as i32) * zoom;
                let ypixel = y + (yline as i32) * zoom;
                // // A draw a rectangle which almost fills our window with it !
                canvas.fill_rect(Rect::new(xpixel, ypixel, self.zoom_level, self.zoom_level)).unwrap();

            }
        }
    }

}

fn draw<T: RenderTarget>(canvas: &mut Canvas<T>, x: i32, y: i32, zoom_level: u32, tile_row: &TileRowInfo) {
    let v1 = tile_row.low;
    let v2 = tile_row.high;
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


        let zoom = zoom_level as i32;
        let xpixel = x + (xline as i32) * zoom;
        let ypixel = y;
        // // A draw a rectangle which almost fills our window with it !
        canvas.fill_rect(Rect::new(xpixel, ypixel, zoom_level, zoom_level)).unwrap();
    }
}
