use clap::{App, Arg, SubCommand};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{RenderTarget, WindowCanvas};
use sdl2::EventPump;
use std::time::{Duration, Instant};

use nesemu::graphic::Canvas;
use nesemu::{
    cpu::memory::Memory,
    graphic::{EmulatorInput, GraphicHandler},
    joypad::{InputAction, InputState, Player},
    nes::Nes,
    ppu::{memory::RegisterType, palette, Ppu},
    rom,
};
use std::collections::HashMap;

// This is the NES default
const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;

struct SdlCanvas(WindowCanvas);

impl nesemu::graphic::Canvas for SdlCanvas {
    fn set_color(&mut self, color: nesemu::graphic::Color) {
        self.0.set_draw_color(Color::RGB(color.r, color.g, color.b));
    }
    fn clear_state(&mut self) {
        self.0.clear();
    }
    fn show(&mut self) {
        self.0.present();
    }
    // TODO return Result.
    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32) {
        self.0
            .fill_rect(Rect::new(x, y, w, h))
            .expect("Cannot draw rectangle");
    }
}

fn build_default_input_p1() -> HashMap<Keycode, InputAction> {
    let mut m = HashMap::new();
    // first player
    m.insert(Keycode::W, InputAction::UP);
    m.insert(Keycode::S, InputAction::DOWN);
    m.insert(Keycode::A, InputAction::LEFT);
    m.insert(Keycode::D, InputAction::RIGHT);
    m.insert(Keycode::Z, InputAction::START);
    m.insert(Keycode::X, InputAction::SELECT);
    m.insert(Keycode::F, InputAction::A);
    m.insert(Keycode::G, InputAction::B);

    m
}

fn build_default_input_p2() -> HashMap<Keycode, InputAction> {
    let mut m = HashMap::new();
    m.insert(Keycode::I, InputAction::UP);
    m.insert(Keycode::K, InputAction::DOWN);
    m.insert(Keycode::J, InputAction::LEFT);
    m.insert(Keycode::L, InputAction::RIGHT);
    m.insert(Keycode::N, InputAction::START);
    m.insert(Keycode::M, InputAction::SELECT);
    m.insert(Keycode::O, InputAction::A);
    m.insert(Keycode::P, InputAction::B);

    m
}

pub struct Graphics {
    pub zoom_level: i32,
    //sdl_context: Sdl,
    //video_subsystem: VideoSubsystem,
    canvas: SdlCanvas,
    event_pump: EventPump,
    colors: HashMap<u8, nesemu::graphic::Color>,

    input_map_p1: HashMap<Keycode, InputAction>,
    input_map_p2: HashMap<Keycode, InputAction>,
}

impl Graphics {
    pub fn new(zoom_level: i32) -> Result<Graphics, String> {
        let sdl_context = sdl2::init().map_err(|err| err.to_string())?;
        let video_subsystem = sdl_context.video().map_err(|err| err.to_string())?;

        let width = WIDTH * (zoom_level as u32); //*2;
        let window = video_subsystem
            .window("NES emulator", width, HEIGHT * (zoom_level as u32))
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
            canvas: SdlCanvas(canvas),
            event_pump,
            colors: palette::build_default_colors(),
            input_map_p1: build_default_input_p1(),
            input_map_p2: build_default_input_p2(),
        })
    }
}

impl GraphicHandler for Graphics {
    // This is called in the main loop and will listen for
    // input pressed. If a key is pressed, it will modify
    // the register accordingly.
    fn poll_events(&mut self) -> Vec<EmulatorInput> {
        let mut emu_events = vec![];
        for event in self.event_pump.poll_iter() {
            match event {
                // LEAVE THE EMULATOR
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    emu_events.push(EmulatorInput::QUIT);
                }
                // PAUSE
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    emu_events.push(EmulatorInput::PAUSE);
                }
                // DEBUG mode
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    emu_events.push(EmulatorInput::DEBUG);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    ..
                } => emu_events.push(EmulatorInput::SAVE),

                // NES INPUT
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(action) = self.input_map_p1.get(&keycode) {
                        emu_events.push(EmulatorInput::INPUT(
                            Player::One,
                            *action,
                            InputState::Pressed,
                        ));
                    }

                    if let Some(action) = self.input_map_p2.get(&keycode) {
                        emu_events.push(EmulatorInput::INPUT(
                            Player::Two,
                            *action,
                            InputState::Pressed,
                        ))
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(action) = self.input_map_p1.get(&keycode) {
                        emu_events.push(EmulatorInput::INPUT(
                            Player::One,
                            *action,
                            InputState::Released,
                        ));
                    }

                    if let Some(action) = self.input_map_p2.get(&keycode) {
                        emu_events.push(EmulatorInput::INPUT(
                            Player::Two,
                            *action,
                            InputState::Released,
                        ));
                    }
                }

                _ => {}
            }
        }
        emu_events
    }

    fn display(&mut self, nes: &mut Nes) {
        nes.display(&mut self.canvas, &self.colors, self.zoom_level);
    }

    // For debug !
    //    pub fn draw_debug(&mut self, memory: &Memory){
    //	self.canvas.set_draw_color(palette::get_bg_color(&memory.ppu_mem.palettes, &self.colors));
    //        self.canvas.clear();
    //	// begin at
    //	// X = WIDTH*zoom_level + 10
    //	// Y = 10
    //	// First draw 2 nametables in memory, then we take care of mirroring
    //        let pattern_table_nb = usize::from((memory.ppu_mem.peek(RegisterType::PPUCTRL) >> 4) & 1);
    //	let pattern_table = &memory.get_pattern_table(pattern_table_nb);
    //
    //	let x1 = WIDTH*self.zoom_level + 10;
    //	let x2 = WIDTH*self.zoom_level + 20 + WIDTH;
    //        let nametable1 = &memory.get_logical_table(0);
    //        let nametable2 = &memory.get_logical_table(1);
    //        let nametable3 = &memory.get_logical_table(2);
    //        let nametable4 = &memory.get_logical_table(3);
    //	self.draw_nametable(nametable1, pattern_table, &memory.ppu_mem.palettes, x1 as i32, 10);
    //	self.draw_nametable(nametable2, pattern_table, &memory.ppu_mem.palettes, x2 as i32, 10);
    //	self.draw_nametable(nametable3, pattern_table, &memory.ppu_mem.palettes, x1 as i32, 20+HEIGHT as i32);
    //	self.draw_nametable(nametable4, pattern_table, &memory.ppu_mem.palettes, x2 as i32, 20+HEIGHT as i32);
    //        self.canvas.present();
    //    }
    //
    //    fn draw_nametable(&mut self, nametable: &[u8], pattern_table: &[u8], palettes: &[u8], x: i32, y: i32) {
    //	for row in 0..30i32 {
    //	    let rowattr = row / 4;
    //	    for col in 0..32i32 {
    //		let index = 32*(row as usize) + (col as usize);
    //		let tile = Tile::new(pattern_table, nametable[index] as usize);
    //
    //		let xtile = col*8;
    //		let ytile = row*8;
    //
    //		// fetch attributes for this tile.
    //		let colattr = col / 4;
    //		let attr_idx = 0x3c0 + 8*rowattr+colattr;
    //		let attr_byte = nametable[attr_idx as usize];
    //
    //		let box_row = (row%4) / 2;
    //		let box_col = (col%4) / 2;
    //		let attribute = match (box_row, box_col) {
    //		    (0, 0) => attr_byte & 0b11,
    //		    (0, 1) => (attr_byte & 0b1100) >> 2 ,
    //		    (1, 0) => (attr_byte & 0b110000) >> 4,
    //		    (1, 1) => (attr_byte & 0b11000000) >> 6,
    //		    _ => panic!("Not possible"),
    //		};
    //
    //		let palette = palette::get_bg_palette(attribute, palettes, &self.colors)
    //                    .expect("Cannot get palette from attribute");
    //
    //		// Now draw
    //		tile.draw(&mut self.canvas, x+xtile, y+ytile, &palette);
    //	    }
    //	}
    //    }
}
//
//struct Tile {
//    plane1: [u8; 8],
//    plane2: [u8; 8],
//}
//
//impl Tile {
//    fn new(pattern_table: &[u8], sprite_nb: usize) -> Tile {
//        let mut plane1 = [0; 8];
//        let mut plane2 = [0; 8];
//
//        for i in 0..8 {
//            plane1[i] = pattern_table[16 * sprite_nb + i];
//            plane2[i] = pattern_table[16 * sprite_nb + i + 8];
//        }
//
//        Tile { plane1, plane2 }
//    }
//
//    fn draw<T: RenderTarget>(
//        &self,
//        canvas: &mut Canvas<T>,
//        x: i32,
//        y: i32,
//        palette: &palette::Palette,
//    ) {
//        for yline in 0..8 {
//            let v1 = self.plane1[yline];
//            let v2 = self.plane2[yline];
//            for xline in 0..8 {
//                let bit1 = (v1 >> 8 - (xline + 1)) & 1;
//                let bit2 = ((v2 >> 8 - (xline + 1)) & 1) << 1;
//                let v = bit1 + bit2;
//                if v == 1 {
//                    canvas.set_draw_color(palette.color1);
//                } else if v == 2 {
//                    canvas.set_draw_color(palette.color2);
//                } else if v == 3 {
//                    canvas.set_draw_color(palette.color3);
//                } else {
//                    canvas.set_draw_color(Color::RGB(0, 0, 0));
//                }
//
//                let xpixel = x + (xline as i32);
//                let ypixel = y + (yline as i32);
//                // // A draw a rectangle which almost fills our window with it !
//                canvas
//                    .fill_rect(Rect::new(xpixel, ypixel, 1, 1))
//                    .expect("In Tile::draw, cannot fill_rect");
//            }
//        }
//    }
//}
//
fn run_rom(path: String) {
    let ines = rom::read(path).unwrap();
    let mut nes = Nes::new(ines).unwrap();
    main_loop(nes).unwrap();
}

fn load_state(path: String) {
    let mut nes = Nes::load_state(path).unwrap();
    main_loop(nes).unwrap();
}

fn main_loop(mut nes: Nes) -> Result<(), &'static str> {
    let mut ui = Graphics::new(3).unwrap();
    // Fixed time stamp for input polling.
    let fixed_time_stamp = Duration::new(0, 16666667);
    let mut previous_clock = Instant::now();
    let mut accumulator = Duration::new(0, 0);

    while nes.should_run {
        // Update CPU and PPU (and later APU)
        // if !is_pause {
        nes.tick(nes.is_debug)?;

        // handle events and draw at 60 fps
        while accumulator > fixed_time_stamp {
            accumulator -= fixed_time_stamp;
            let events = ui.poll_events();
            nes.handle_events(events);
            ui.display(&mut nes);
        }

        accumulator += Instant::now() - previous_clock;
        previous_clock = Instant::now();
    }

    Ok(())
}

fn main() {
    let matches = App::new("My Super Program")
        .version("1.0")
        .subcommand(
            SubCommand::with_name("run")
                .about("Run emulator with ROM file")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .help("Path of the ROM file")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("load")
                .about("Load emulator state from file")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .help("Path of the state file")
                        .required(true)
                        .takes_value(true),
                ),
        )
        .get_matches();

    env_logger::init();
    if let Some(matches) = matches.subcommand_matches("run") {
        let rom_path = matches.value_of("input").unwrap();
        run_rom(rom_path.to_string());
    } else if let Some(matches) = matches.subcommand_matches("load") {
        let state_path = matches.value_of("input").unwrap();
        load_state(state_path.to_string());
    } else {
        panic!("Should use run or load subcommand");
    }
}
