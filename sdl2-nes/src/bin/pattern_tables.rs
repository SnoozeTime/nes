use nesemu::rom;
use std::env;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use std::time::Duration;

struct Sprite {
    plane1: [u8; 8],
    plane2: [u8; 8],
}

impl Sprite {
    fn new(pattern_table: &[u8], sprite_nb: usize) -> Sprite {
        let mut plane1 = [0; 8];
        let mut plane2 = [0; 8];

        for i in 0..8 {
            plane1[i] = pattern_table[16 * sprite_nb + i];
            plane2[i] = pattern_table[16 * sprite_nb + i + 8];
        }

        Sprite { plane1, plane2 }
    }

    fn draw<T: RenderTarget>(&self, canvas: &mut Canvas<T>, x: i32, y: i32) {
        for yline in 0..8 {
            let v1 = self.plane1[yline];
            let v2 = self.plane2[yline];
            for xline in 0..8 {
                let bit1 = (v1 >> 8 - (xline + 1)) & 1;
                let bit2 = ((v2 >> 8 - (xline + 1)) & 1) << 1;
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
                    canvas
                        .fill_rect(Rect::new(x + xline as i32 * 5, y + yline as i32 * 5, 5, 5))
                        .unwrap();
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

    draw(ines);
}

fn draw(ines: rom::INesFile) {
    let chr_rom = ines.get_chr_rom(1).unwrap();
    let sprites_left: Vec<Sprite> = (0..256).map(|i| Sprite::new(&chr_rom, i)).collect();
    let sprites_right: Vec<Sprite> = (256..512).map(|i| Sprite::new(&chr_rom, i)).collect();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("NES emulator", 1280, 640)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                _ => {}
            }
        }
        let mut x = 0;
        let mut y = 0;
        for sprite in &sprites_left {
            sprite.draw(&mut canvas, x, y);
            x = (x + 40) % 640;
            if x == 0 {
                y += 40;
            }
        }

        x = 640;
        y = 0;
        for sprite in &sprites_right {
            sprite.draw(&mut canvas, x, y);
            x = ((x + 40) % 640) + 640;
            if x == 640 {
                y += 40;
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
