extern crate sdl2;
use std::env;
mod nes;
mod cpu;
mod rom;
mod ppu;
use cpu::cpu::Cpu;
use nes::Nes;


use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::rect::{Point, Rect};
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::render::{RenderTarget, Canvas};

struct Sprite {
    plane1: [u8; 8],
    plane2: [u8; 8],
}

impl Sprite {
    fn new(pattern_table: &[u8], sprite_nb: usize) -> Sprite {

        let mut plane1 = [0;8];
        let mut plane2 = [0;8];

        for i in 0..8 {
            plane1[i] = pattern_table[16*sprite_nb + i];
            plane2[i] = pattern_table[16*sprite_nb + i + 8];
        }

        Sprite { plane1, plane2 }
    }

    fn draw<T: RenderTarget>(&self, canvas: &mut Canvas<T>, x: i32, y: i32) {

        for yline in 0..8 {
            let v1 = self.plane1[yline];
            let v2 = self.plane2[yline];
            for xline in 0..8 {
                let bit1 = (v1 >> 8-(xline+1)) & 1;
                let bit2 = ((v2 >> 8-(xline+1)) & 1) << 1;
                let v = bit1 + bit2;
                if v > 0 {
        // change the color of our drawing with a gold-color ...
                if v == 1 {
                canvas.set_draw_color(Color::RGB(255, 0, 0));
                } else if v == 2 {
                canvas.set_draw_color(Color::RGB(0, 255, 0));
                } else {
                canvas.set_draw_color(Color::RGB(0, 0, 255));
                }
                 // // A draw a rectangle which almost fills our window with it !
        canvas.fill_rect(Rect::new(x + xline as i32 *5, y + yline as i32 *5, 5, 5));
                }
            }
        }
    }

}

pub fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage {} <FILENAME>", args[0]);
    }

    let name = args[1].clone();
    let ines = rom::read(name).expect("IIIIINNNNNEEESS");


    let mut nes = Nes::new(ines);
    nes.run();
}

