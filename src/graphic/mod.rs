use super::cpu::memory::Memory;
use super::ppu::{Ppu, TileRowInfo, SpriteInfo};
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
    pub fn display(&mut self, memory: &Memory, ppu: &mut Ppu) {

        if ppu.should_display() {
            self.canvas.set_draw_color(palette::get_bg_color(&memory.ppu_mem.ppu_mem, &self.colors));
            self.canvas.clear();
            let mut index: usize = 0;
            for row in 0..240i32 {
                for col in 0..32i32 {
                    index = 32*(row as usize) + (col as usize);
                    let tilerow = &ppu.virtual_buffer[index];
                    let xtile = col*8*(self.zoom_level as i32);
                    let ytile = row*(self.zoom_level as i32);

                    let box_row = (row/8 % 4) / 2;
                    let box_col = (col%4) / 2;
                    let attribute = match (box_row, box_col) {
                        (0, 0) => tilerow.attr & 0b11,
                        (0, 1) => (tilerow.attr & 0b1100) >> 2 ,
                        (1, 0) => (tilerow.attr & 0b110000) >> 4,
                        (1, 1) => (tilerow.attr & 0b11000000) >> 6,
                        _ => panic!("Not possible"),
                    };

                    let palette = palette::get_bg_palette(attribute, &memory.ppu_mem.ppu_mem, &self.colors).unwrap();                   
                    self.draw_tilerow(xtile, ytile, &tilerow, &palette);
                }
            }

            // now the sprites.
            let sprites = &ppu.virtual_sprite_buffer;
            for sprite in sprites {
                self.draw_sprite(sprite, memory);
            }

            self.canvas.present();
        }
    }

    fn draw_tilerow(&mut self, x: i32, y: i32, tile_row: &TileRowInfo, palette: &palette::Palette) {
        let v1 = tile_row.low;
        let v2 = tile_row.high;
        for xline in 0..8 {
            let bit1 = (v1 >> 8-(xline+1)) & 1;
            let bit2 = ((v2 >> 8-(xline+1)) & 1) << 1;
            let v = bit1 + bit2;

            if v > 0 {
                if v == 1 {
                    self.canvas.set_draw_color(palette.color1);
                } else if v == 2 {
                    self.canvas.set_draw_color(palette.color2);
                } else if v == 3 {
                    self.canvas.set_draw_color(palette.color3);
                } else {
                    self.canvas.set_draw_color(palette.background);
                }

                let zoom = self.zoom_level as i32;
                let xpixel = x + (xline as i32) * zoom;
                let ypixel = y;
                self.canvas.fill_rect(Rect::new(xpixel,
                                                ypixel,
                                                self.zoom_level,
                                                self.zoom_level)).unwrap();
            }
        }

    }

    fn draw_sprite(&mut self, sprite: &SpriteInfo, memory: &Memory) {
        let x = sprite.x as i32 * self.zoom_level as i32;
        let y = sprite.y as i32 * self.zoom_level as i32;

        let palette = palette::get_bg_palette(1, &memory.ppu_mem.ppu_mem, &self.colors).unwrap();
        self.draw_tilerow(x, y, &sprite.tile, &palette);
    }
}

