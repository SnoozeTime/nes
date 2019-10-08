use std::path::PathBuf;
use structopt::StructOpt;
use tracing::{error, info, info_span, trace};
#[macro_use]
extern crate glium;
use glium::glutin;
use glium::glutin::{ElementState, VirtualKeyCode};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use nesemu::{
    graphic::EmulatorInput,
    joypad::{InputAction, InputState, Player},
    nes::Nes,
    rom,
};
mod graphics;
mod ui;
use ui::{AppState, Application, UiEvent};
mod audio;

fn build_default_input_p1() -> HashMap<VirtualKeyCode, InputAction> {
    let mut m = HashMap::new();
    // first player
    m.insert(VirtualKeyCode::W, InputAction::UP);
    m.insert(VirtualKeyCode::S, InputAction::DOWN);
    m.insert(VirtualKeyCode::A, InputAction::LEFT);
    m.insert(VirtualKeyCode::D, InputAction::RIGHT);
    m.insert(VirtualKeyCode::Z, InputAction::START);
    m.insert(VirtualKeyCode::X, InputAction::SELECT);
    m.insert(VirtualKeyCode::F, InputAction::A);
    m.insert(VirtualKeyCode::G, InputAction::B);

    m
}

fn build_default_input_p2() -> HashMap<VirtualKeyCode, InputAction> {
    let mut m = HashMap::new();
    m.insert(VirtualKeyCode::I, InputAction::UP);
    m.insert(VirtualKeyCode::K, InputAction::DOWN);
    m.insert(VirtualKeyCode::J, InputAction::LEFT);
    m.insert(VirtualKeyCode::L, InputAction::RIGHT);
    m.insert(VirtualKeyCode::N, InputAction::START);
    m.insert(VirtualKeyCode::M, InputAction::SELECT);
    m.insert(VirtualKeyCode::O, InputAction::A);
    m.insert(VirtualKeyCode::P, InputAction::B);

    m
}

const CPU_CYCLES_PER_FRAME: u64 = 29_780;

macro_rules! timed_block {
    ($content:expr, $e:expr) => {
        {
            let start = Instant::now();
            let span = info_span!($content, start = ?start);
            let _enter = span.enter();

            $e;

            let end = Instant::now();
            trace!(end = ?end, duration= ?(end - start));
        }

    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "NES emulator (glium version)", about = "NES emulator with GUI")]
struct Opt {
    /// Can provide the rom from the CLI
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,

    /// Record all the audio to a Wav file.
    #[structopt(short = "w", parse(from_os_str))]
    recording_name: Option<PathBuf>,

    /// If present, won't play sound
    #[structopt(long = "no-sound")]
    no_sound: bool,

    /// Choose the palette file. Will use default palette if absent.
    #[structopt(long = "palette", parse(from_os_str))]
    palette: Option<PathBuf>,
}

fn main() {
    let sub = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(sub).unwrap();

    let opt = Opt::from_args();
    info!("Will start with {:?}", opt);

    // 1. INITIALIZE BASIC SYSTEMS (AUDIO + GRAPHICS)
    // ----------------------------------------------------------
    let mut audio = if let Some(recording_name) = opt.recording_name {
        audio::AudioSystem::with_recording(recording_name)
    } else {
        audio::AudioSystem::init()
    }
    .expect("Cannot initialize audio system");

    if !opt.no_sound {
        audio.resume();
    }

    let mut events_loop = glutin::EventsLoop::new();
    let mut graphic_system = graphics::GraphicSystem::init(opt.palette, &events_loop)
        .expect("Cannot initialize graphic system");

    // 2. INITIALIZE APPLICATION STATE
    // ------------------------------------------------------------
    let input_map_p1 = build_default_input_p1();
    let input_map_p2 = build_default_input_p2();
    let mut application = Application::default();
    let fixed_time_stamp = Duration::new(0, 16666667);

    // 3. CREATE EMULATOR
    // ------------------------------------------------------------
    // now load the nes emulator.
    let mut nes = if let Some(rom) = opt.input {
        let ines = rom::read(rom).unwrap();
        let nes = Nes::new(ines).unwrap();
        application.set_state(AppState::Running);
        nes
    } else {
        Nes::empty()
    };

    // 4. MAIN LOOP
    // -----------------------------------------------------------
    while application.should_run() {
        let now = Instant::now();

        // ONE NES FRAME
        // -------------------------------------------------
        timed_block!("NES frame", {
            if let AppState::Running = application.current_state() {
                let mut total_cycles = CPU_CYCLES_PER_FRAME;
                while total_cycles > 0 {
                    total_cycles = total_cycles.saturating_sub(nes.tick(false).unwrap());
                }
            }
        });
        // DISPLAY
        // --------------------------------------------------
        timed_block!("Display", {
            // NES buffer
            // ----------------------
            if nes.should_display() {
                if let Err(e) = graphic_system.render_nes_frame(&nes) {
                    error!("{}", e);
                }
            }

            graphic_system.render(|ui| match ui::run_ui(&ui, &mut application) {
                Some(UiEvent::LoadRom) => {
                    // If can find a rom, load it. Otherwise, restore state before
                    // opening the file explorer.
                    if let Some(rom) = application.rom_name() {
                        let ines = rom::read(rom).unwrap();
                        nes = Nes::new(ines).unwrap();
                        application.set_state(AppState::Running);
                    } else {
                        application.reset_to_previous();
                    }
                }
                Some(UiEvent::Resume) => application.reset_to_previous(),
                Some(UiEvent::SaveState) => {
                    if let Err(e) = nes.save_state() {
                        println!("Error while saving {} = {}", nes.get_save_name(), e);
                    }
                }
                Some(UiEvent::LoadState) => {
                    if let Ok(new_nes) = Nes::load_state(nes.get_save_name()) {
                        nes = new_nes;
                    } else {
                        println!("Could not load {}", nes.get_save_name());
                    }
                }

                _ => (),
            });
        });

        // AUDIO
        // --------------------------------------------------------
        timed_block!("Process audio", {
            let samples = nes.audio_samples();
            if let Err(e) = audio.process_samples(&samples) {
                error!("something happened when processing audio samples = {}", e);
            }
        });

        // EVENT HANDLING
        // --------------------------------------------------------
        timed_block!("Process events", {
            let mut emu_events = vec![];
            events_loop.poll_events(|ev| {
                graphic_system.handle_imgui_events(&ev);

                //platform.handle_event(imgui.io_mut(), &window, &ev);
                match ev {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => application.exit(),
                        glutin::WindowEvent::KeyboardInput { input, .. } => {
                            if let Some(key) = input.virtual_keycode {
                                if ElementState::Pressed == input.state {
                                    if let Some(action) = input_map_p1.get(&key) {
                                        emu_events.push(EmulatorInput::INPUT(
                                            Player::One,
                                            *action,
                                            InputState::Pressed,
                                        ));
                                    }

                                    if let Some(action) = input_map_p2.get(&key) {
                                        emu_events.push(EmulatorInput::INPUT(
                                            Player::Two,
                                            *action,
                                            InputState::Pressed,
                                        ));
                                    }
                                } else {
                                    if let Some(action) = input_map_p1.get(&key) {
                                        emu_events.push(EmulatorInput::INPUT(
                                            Player::One,
                                            *action,
                                            InputState::Released,
                                        ));
                                    }
                                    if let Some(action) = input_map_p2.get(&key) {
                                        emu_events.push(EmulatorInput::INPUT(
                                            Player::Two,
                                            *action,
                                            InputState::Released,
                                        ));
                                    }
                                }
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                }
            });
            nes.handle_events(emu_events);
        });

        // FIXED TIME STEP
        let dt = Instant::now() - now;
        if dt < fixed_time_stamp {
            std::thread::sleep(fixed_time_stamp - dt);
        } else {
        }
    }
}
