use tracing::{info_span, trace};
#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;
use glium::glutin;
use glium::glutin::{ElementState, VirtualKeyCode};
use glium::Surface;
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, MenuItem, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::collections::HashMap;
use std::io::Cursor;
use std::time::{Duration, Instant};

mod ui;
use ui::{AppState, FileExplorer, UiEvent};

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

use nesemu::{
    graphic::EmulatorInput,
    joypad::{InputAction, InputState, Player},
    nes::Nes,
    ppu::palette,
    rom,
};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2], // <- this is new
}

const CPU_CYCLES_PER_FRAME: u64 = 29_780;
implement_vertex!(Vertex, position, tex_coords);

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
fn main() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_specs = sdl2::audio::AudioSpecDesired {
        freq: Some(44100),
        samples: Some(1024),
        channels: Some(1),
    };
    let audio = audio_subsystem
        .open_queue::<i16, _>(None, &desired_specs)
        .unwrap();
    audio.resume();

    let input_map_p1 = build_default_input_p1();
    let input_map_p2 = build_default_input_p2();
    let sub = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(sub).unwrap();

    let image = image::load(
        Cursor::new(&include_bytes!("../tuto-06-texture.png")[..]),
        image::PNG,
    )
    .unwrap()
    .to_rgba();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_vsync(false);
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
    }

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
        FontSource::TtfData {
            data: include_bytes!("../resources/mplus-1p-regular.ttf"),
            size_pixels: font_size,
            config: Some(FontConfig {
                rasterizer_multiply: 1.75,
                glyph_ranges: FontGlyphRanges::japanese(),
                ..FontConfig::default()
            }),
        },
    ]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let mut renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    let mut texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let ratio = 1.0; // 16.0 / 15.0;
    let vertex_buffer = glium::VertexBuffer::new(
        &display,
        &[
            Vertex {
                position: [-0.5 * ratio, -0.5],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [0.5 * ratio, -0.5],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [0.5 * ratio, 0.5],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [-0.5 * ratio, 0.5],
                tex_coords: [0.0, 1.0],
            },
        ],
    )
    .unwrap();

    let indices = glium::index::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &[0, 1, 2, 0, 2, 3u16][..],
    )
    .unwrap();

    let vertex_shader_src = r#"
    #version 140

    in vec2 position;

in vec2 tex_coords;
out vec2 v_tex_coords;

    void main() {
        v_tex_coords = tex_coords;
        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;

    let fragment_shader_src = r#"
    #version 140


in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;

    void main() {
        color = texture(tex, v_tex_coords);
    }
"#;
    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();
    let mut closed = false;

    let fixed_time_stamp = Duration::new(0, 16666667);

    // now load the nes emulator.
    let ines = rom::read(String::from("../games/megaman2.nes")).unwrap();
    let mut nes = Nes::empty();

    let colors = palette::build_default_colors();

    let gl_window = display.gl_window();
    let window = gl_window.window();
    let mut last_frame = Instant::now();

    let mut is_nes_running = false;
    let mut file_explorer = FileExplorer::default();
    let mut previous_state = AppState::Nothing;
    let mut app_state = AppState::Nothing;
    while !closed {
        let now = Instant::now();

        // ONE NES FRAME
        // -------------------------------------------------
        timed_block!("NES frame", {
            if let AppState::Running = app_state {
                let mut total_cycles = CPU_CYCLES_PER_FRAME;
                while total_cycles > 0 {
                    total_cycles = total_cycles.saturating_sub(nes.tick(false).unwrap());
                }
            }
        });
        // DISPLAY
        // --------------------------------------------------
        timed_block!("Display", {
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);
            // NES buffer
            // ----------------------
            if nes.should_display() {
                let mut frame = image::ImageBuffer::new(256, 240);
                let dimensions = frame.dimensions();
                for x in 0..256u32 {
                    for y in 0..240u32 {
                        let pixel = nes.get_pixel(y as usize, x as usize) as usize;
                        let color = colors[pixel];
                        let pixel = frame.get_pixel_mut(x, y);
                        *pixel = image::Rgb([color.r, color.g, color.b]);
                    }
                }

                let image = glium::texture::RawImage2d::from_raw_rgb_reversed(
                    &frame.into_raw(),
                    dimensions,
                );
                texture = glium::texture::Texture2d::new(&display, image).unwrap();
            }

            let uniforms = uniform! {
                 tex: texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest).minify_filter(glium::uniforms::MinifySamplerFilter::Nearest),
            };
            target
                .draw(
                    &vertex_buffer,
                    &indices,
                    &program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();

            // IMGUI
            // --------------------
            let io = imgui.io_mut();
            platform
                .prepare_frame(io, &window)
                .expect("Failed to start frame");
            last_frame = io.update_delta_time(last_frame);
            let ui = imgui.frame();
            match ui::run_ui(&ui, &mut app_state, &mut previous_state, &mut file_explorer) {
                Some(UiEvent::LoadRom) => {
                    // If can find a rom, load it. Otherwise, restore state before
                    // opening the file explorer.
                    if let Some(ref rom) = file_explorer.selected {
                        let ines = rom::read(rom).unwrap();
                        nes = Nes::new(ines).unwrap();
                        app_state = AppState::Running;
                    } else {
                        app_state = previous_state;
                    }
                }
                Some(UiEvent::Resume) => app_state = previous_state,
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
            }
            platform.prepare_render(&ui, &window);
            let draw_data = ui.render();
            renderer
                .render(&mut target, draw_data)
                .expect("Rendering failed");

            target.finish().unwrap();
        });

        // AUDIO
        // --------------------------------------------------------
        let samples = nes.audio_samples();
        audio.queue(&samples);

        // EVENT HANDLING
        // --------------------------------------------------------
        timed_block!("Process events", {
            let mut emu_events = vec![];
            events_loop.poll_events(|ev| {
                platform.handle_event(imgui.io_mut(), &window, &ev);
                match ev {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::CloseRequested => closed = true,
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
        let dt = Instant::now() - now;
        if dt < fixed_time_stamp {
            std::thread::sleep(fixed_time_stamp - dt);
        } else {
        }
    }
}
