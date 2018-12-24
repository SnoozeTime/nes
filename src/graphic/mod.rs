use super::cpu::memory::Memory;
use super::ppu::{Ppu, TileRowInfo, SpriteInfo};
use super::ppu::memory::RegisterType;
use super::ppu::palette;
use std::collections::HashMap;
/// Uses SDL 2 to render graphics.

extern crate sdl2;
use self::sdl2::EventPump;
use self::sdl2::pixels::Color;
use self::sdl2::event::Event;
use self::sdl2::rect::Rect;
use self::sdl2::keyboard::Keycode;
use self::sdl2::render::{RenderTarget, Canvas, WindowCanvas};

use joypad::InputAction;

// This is the NES default
const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;

// Emulator specific action
pub enum EmulatorInput {
    PAUSE,
    QUIT,
}

fn build_default_input() -> HashMap<Keycode, InputAction> {

    let mut m = HashMap::new();
    m.insert(Keycode::W, InputAction::UP);
    m.insert(Keycode::S, InputAction::DOWN);
    m.insert(Keycode::A, InputAction::LEFT);
    m.insert(Keycode::D, InputAction::RIGHT);
    m.insert(Keycode::B, InputAction::START);
    m.insert(Keycode::V, InputAction::SELECT);
    m.insert(Keycode::K, InputAction::A);
    m.insert(Keycode::L, InputAction::B);
    m
}

pub struct Graphics {
    zoom_level: u32,
    //sdl_context: Sdl,
    //video_subsystem: VideoSubsystem,
    canvas: WindowCanvas,
    event_pump: EventPump,
    colors: HashMap<u8, Color>,

    input_map: HashMap<Keycode, InputAction>,
}


impl Graphics {

    pub fn new(zoom_level: u32) -> Result<Graphics, String> {
	let sdl_context = sdl2::init()
	    .map_err(|err| err.to_string())?;
	let video_subsystem = sdl_context.video()
	    .map_err(|err| err.to_string())?;

	let width = WIDTH*zoom_level*2;
	let window = video_subsystem
	    .window("NES emulator", width, HEIGHT*zoom_level)
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
	    canvas,
	    event_pump,
	    colors: palette::build_default_colors(),
	    input_map: build_default_input(),
	})
    }

    // This is called in the main loop and will listen for 
    // input pressed. If a key is pressed, it will modify
    // the register accordingly.
    pub fn handle_events(&mut self, mem: &mut Memory, is_paused: bool) -> Option<EmulatorInput> {
	for event in self.event_pump.poll_iter() {
	    match event {
		// LEAVE THE EMULATOR
		Event::Quit{..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..}
		=> {
		    return Some(EmulatorInput::QUIT);
		},
		// PAUSE
		Event::KeyDown {keycode: Some(Keycode::Space), .. } => {
		    return Some(EmulatorInput::PAUSE);
		},

		// NES INPUT
		Event::KeyDown { keycode: Some(keycode), ..} => {
		    if let Some(action) = self.input_map.get(&keycode) {
			if !is_paused {
			    mem.joypad.button_down(action);
			}
		    }
		},
		Event::KeyUp { keycode: Some(keycode), ..} => {
		    if let Some(action) = self.input_map.get(&keycode) {
			if !is_paused {
			    mem.joypad.button_up(action);
			}
		    }
		},

		_ => {
		},
	    }
	}
	None
    }


    pub fn display(&mut self, memory: &Memory, ppu: &mut Ppu) {

	if ppu.should_display() {
	    self.canvas.set_draw_color(palette::get_bg_color(&memory.ppu_mem.ppu_mem, &self.colors));
	    self.canvas.clear();
	    for row in 0..240i32 {
		for col in 0..32i32 {
		    let index = 32*(row as usize) + (col as usize);
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

		    let palette = palette::get_bg_palette(attribute, &memory.ppu_mem.ppu_mem, &self.colors).expect("Cannot get palette for background");                   
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
						self.zoom_level))
                    .expect("In draw_tilerow, cannot fill_rect");
	    }
	}

    }

    fn draw_sprite(&mut self, sprite: &SpriteInfo, memory: &Memory) {
	let x = sprite.x as i32 * self.zoom_level as i32;
	let y = sprite.y as i32 * self.zoom_level as i32;

	let palette = palette::get_sprite_palette(
	    sprite.tile.attr & 0b11, &memory.ppu_mem.ppu_mem, &self.colors)
            .expect("In draw-sprite, cannot get sprite_palette");
	self.draw_tilerow(x, y, &sprite.tile, &palette);
    }

    // For debug !
    pub fn draw_debug(&mut self, memory: &Memory){
	self.canvas.set_draw_color(palette::get_bg_color(&memory.ppu_mem.ppu_mem, &self.colors));
        self.canvas.clear();
	// begin at
	// X = WIDTH*zoom_level + 10
	// Y = 10
	// First draw 2 nametables in memory, then we take care of mirroring
	let pattern_table_addr = 0x1000 *
	    ((memory.ppu_mem.peek(RegisterType::PPUCTRL) >> 4) & 1) as usize;
	let pattern_table = &memory.ppu_mem.ppu_mem[pattern_table_addr..pattern_table_addr+0x1000]; 

	let x1 = WIDTH*self.zoom_level + 10;
	let x2 = WIDTH*self.zoom_level + 20 + WIDTH;
        let nametable1 = &memory.ppu_mem.get_logical_table(0);
        let nametable2 = &memory.ppu_mem.get_logical_table(1);
        let nametable3 = &memory.ppu_mem.get_logical_table(2);
        let nametable4 = &memory.ppu_mem.get_logical_table(3);
	self.draw_nametable(nametable1, pattern_table, &memory.ppu_mem.ppu_mem, x1 as i32, 10);
	self.draw_nametable(nametable2, pattern_table, &memory.ppu_mem.ppu_mem, x2 as i32, 10);
	self.draw_nametable(nametable3, pattern_table, &memory.ppu_mem.ppu_mem, x1 as i32, 20+HEIGHT as i32);
	self.draw_nametable(nametable4, pattern_table, &memory.ppu_mem.ppu_mem, x2 as i32, 20+HEIGHT as i32);
        self.canvas.present();
    }

    fn draw_nametable(&mut self, nametable: &[u8], pattern_table: &[u8], ppu_mem: &[u8], x: i32, y: i32) {
	for row in 0..30i32 {
	    let rowattr = row / 4;
	    for col in 0..32i32 {
		let index = 32*(row as usize) + (col as usize);
		let tile = Tile::new(pattern_table, nametable[index] as usize);

		let xtile = col*8;
		let ytile = row*8;

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

		let palette = palette::get_bg_palette(attribute, ppu_mem, &self.colors)
                    .expect("Cannot get palette from attribute");

		// Now draw
		tile.draw(&mut self.canvas, x+xtile, y+ytile, &palette);
	    }
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

	Tile { plane1, plane2}
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
		    canvas.set_draw_color(Color::RGB(0,0,0));
		}

		let xpixel = x + (xline as i32);
		let ypixel = y + (yline as i32);
		// // A draw a rectangle which almost fills our window with it !
		canvas.fill_rect(Rect::new(xpixel, ypixel, 1, 1))
                    .expect("In Tile::draw, cannot fill_rect");
	    }
	}
    }

}

