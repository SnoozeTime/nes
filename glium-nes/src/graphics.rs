//! Module that sets up the OpenGL and ImGUI renderers.
//!
//! OpenGL is abstracted with glium library. The code is actually simple and could
//! be replaced by vulkan or other graphic library quite easily. A quad is drawn
//! and the NES frame is applied as a texture.

use glium::backend::glutin::DisplayCreationError;
use glium::glutin;
use glium::Surface;
use snafu::{ResultExt, Snafu};
use std::time::Instant;
use tracing::{error, info};

use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use nesemu::{
    graphic::Color,
    nes::Nes,
    ppu::palette::{build_default_colors, load_palette},
};
use std::io::Cursor;
use std::path::Path;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2], // <- this is new
}
implement_vertex!(Vertex, position, tex_coords);

#[derive(Debug, Snafu)]
pub enum GraphicError {
    #[snafu(display("Cannot create glium display"))]
    CannotCreateDisplay { source: DisplayCreationError },

    #[snafu(display("Cannot create imgui renderer"))]
    CannotCreateImgui {
        source: imgui_glium_renderer::RendererError,
    },

    #[snafu(display("Cannot create image"))]
    CannotCreateImage { source: image::ImageError },

    #[snafu(display("Cannot create texture"))]
    CannotCreateTexture {
        source: glium::texture::TextureCreationError,
    },

    #[snafu(display("Cannot create vertex buffer"))]
    CannotCreateVertexBuffer {
        source: glium::vertex::BufferCreationError,
    },

    #[snafu(display("Cannot create index buffer"))]
    CannotCreateIndexBuffer {
        source: glium::index::BufferCreationError,
    },

    #[snafu(display("Shader error"))]
    CannotCreateProgram {
        source: glium::program::ProgramCreationError,
    },
}

/// Will hold all the data related to glium and imgui.
pub struct GraphicSystem {
    colors: [Color; 64],
    display: glium::Display,

    pub imgui: Context,
    renderer: Renderer,
    pub platform: WinitPlatform,
    last_frame: Instant,

    // then to display
    texture: glium::texture::Texture2d,
    vertices: glium::vertex::VertexBuffer<Vertex>,
    indices: glium::index::IndexBuffer<u16>,
    program: glium::program::Program,
}

impl GraphicSystem {
    pub fn init<P: AsRef<Path>>(
        palette: Option<P>,
        events_loop: &glutin::EventsLoop,
    ) -> Result<Self, GraphicError> {
        info!("Initialize GraphicSystem");

        info!("Initialize glium display");
        let wb = glutin::WindowBuilder::new();
        let cb = glutin::ContextBuilder::new().with_vsync(false);
        let display = glium::Display::new(wb, cb, events_loop).context(CannotCreateDisplay {})?;

        info!("Initialize ImGui");
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

        let renderer = Renderer::init(&mut imgui, &display).context(CannotCreateImgui {})?;

        // --------------------------------------------
        info!("Initialize texture and glium vertex data");
        let image = image::load(
            Cursor::new(&include_bytes!("../tuto-06-texture.png")[..]),
            image::PNG,
        )
        .context(CannotCreateImage {})?
        .to_rgba();
        let image_dimensions = image.dimensions();
        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

        let texture =
            glium::texture::Texture2d::new(&display, image).context(CannotCreateTexture {})?;

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
        .context(CannotCreateVertexBuffer {})?;

        let indices = glium::index::IndexBuffer::new(
            &display,
            glium::index::PrimitiveType::TrianglesList,
            &[0, 1, 2, 0, 2, 3u16][..],
        )
        .context(CannotCreateIndexBuffer {})?;

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
                .context(CannotCreateProgram {})?;
        let last_frame = Instant::now();

        let colors = if let Some(path) = palette {
            let path = path.as_ref();
            match load_palette(&path) {
                Ok(p) => {
                    info!("Will use palette {}", path.display());
                    p
                }
                Err(e) => {
                    error!("Error loading {} = {}", path.display(), e);
                    build_default_colors()
                }
            }
        } else {
            build_default_colors()
        };

        Ok(Self {
            colors,
            last_frame,
            display,
            platform,
            imgui,
            renderer,
            texture,
            vertices: vertex_buffer,
            indices,
            program,
        })
    }

    /// Will render the full NES frame to a texture
    pub fn render_nes_frame(&mut self, nes: &Nes) -> Result<(), GraphicError> {
        let mut frame = image::ImageBuffer::new(256, 240);
        let dimensions = frame.dimensions();
        for x in 0..256u32 {
            for y in 0..240u32 {
                let pixel = nes.get_pixel(y as usize, x as usize) as usize;
                let color = self.colors[pixel];
                let pixel = frame.get_pixel_mut(x, y);
                *pixel = image::Rgb([color.r, color.g, color.b]);
            }
        }

        let image =
            glium::texture::RawImage2d::from_raw_rgb_reversed(&frame.into_raw(), dimensions);
        self.texture =
            glium::texture::Texture2d::new(&self.display, image).context(CannotCreateTexture {})?;

        Ok(())
    }
    pub fn render<F>(&mut self, mut run_ui: F)
    where
        F: FnMut(&mut Ui),
    {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        // First draw the texture
        let uniforms = uniform! {
             tex: self.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest).minify_filter(glium::uniforms::MinifySamplerFilter::Nearest),
        };
        target
            .draw(
                &self.vertices,
                &self.indices,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();

        // Then draw the UI
        let gl_window = self.display.gl_window();
        let window = gl_window.window();

        self.platform
            .prepare_frame(self.imgui.io_mut(), &window)
            .unwrap();
        self.last_frame = self.imgui.io_mut().update_delta_time(self.last_frame);
        let mut ui = self.imgui.frame();
        run_ui(&mut ui);

        self.platform.prepare_render(&ui, &window);
        let draw_data = ui.render();
        self.renderer
            .render(&mut target, draw_data)
            .expect("Rendering failed");

        target.finish().unwrap();
    }

    pub fn handle_imgui_events(&mut self, event: &glium::glutin::Event) {
        let gl_window = self.display.gl_window();
        let window = gl_window.window();
        self.platform
            .handle_event(self.imgui.io_mut(), &window, event);
    }
}
