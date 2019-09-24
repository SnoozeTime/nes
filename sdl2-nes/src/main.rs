use clap::{App, Arg, SubCommand};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{RenderTarget, WindowCanvas};
use sdl2::EventPump;
use std::time::{Duration, Instant};

use nesemu::{
    cpu::memory::Memory,
    graphic::EmulatorInput,
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

impl SdlCanvas {
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
}

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
    let zoom_level = ui.zoom_level;
    while nes.should_run {
        // Update CPU and PPU (and later APU)
        // if !is_pause {
        nes.tick(nes.is_debug)?;

        // handle events and draw at 60 fps
        while accumulator > fixed_time_stamp {
            accumulator -= fixed_time_stamp;
            let events = ui.poll_events();
            nes.handle_events(events);
            //ui.display(&mut nes);

            if nes.should_display() {
                let bg_color = palette::get_bg_color(&nes.memory().ppu_mem.palettes, &ui.colors);
                ui.canvas.set_color(bg_color);
                ui.canvas.clear_state();

                for row in 0..240i32 {
                    for col in 0..256i32 {
                        let pixel = nes.get_pixel(row, col);
                        ui.canvas.set_color(pixel);

                        let xpixel = col * zoom_level;
                        let ypixel = row * zoom_level;

                        ui.canvas
                            .draw_rect(xpixel, ypixel, zoom_level as u32, zoom_level as u32);
                    }
                }
                ui.canvas.show();
            }
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
