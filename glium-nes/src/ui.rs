use imgui::*;
use nesemu::apu::ApuLevels;
use std::borrow::Cow;
use std::default::Default;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tracing::error;

pub struct Application {
    is_running: bool,
    pub is_game_running: bool,

    // File explorer to open a new ROM
    file_explorer_opened: bool,
    file_explorer: FileExplorer,

    // Sound configuration -> levels, relative volume and so on,
    sound_config_opened: bool,
    pub sound_levels: Levels,
    // the one being modified.
    dirty_sound_levels: Levels,
}

#[derive(Debug, Clone, Copy)]
pub struct Levels {
    master: i32,
    pulse_1: i32,
    pulse_2: i32,
    triangle: i32,
}

impl Levels {
    pub fn to_apu_levels(&self) -> ApuLevels {
        let mut levels = ApuLevels::default();
        levels.set_master_level(self.master as f64 * 100.0);
        levels.set_pulse1_level(self.pulse_1 as f64 / 100.0);
        levels.set_pulse2_level(self.pulse_2 as f64 / 100.0);
        levels.set_triangle_level(self.triangle as f64 / 100.0);
        levels
    }
}

impl Default for Levels {
    fn default() -> Self {
        Self {
            master: 100,
            pulse_1: 100,
            pulse_2: 100,
            triangle: 100,
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        Self {
            is_running: true,
            is_game_running: false,
            file_explorer_opened: false,
            file_explorer: FileExplorer::default(),

            sound_config_opened: false,
            sound_levels: Levels::default(),
            dirty_sound_levels: Levels::default(),
        }
    }
}

impl Application {
    /// Return the rom name to load if selected.
    pub fn rom_name(&self) -> Option<&PathBuf> {
        self.file_explorer.selected.as_ref()
    }

    /// Continue to run if returns true
    pub fn should_run(&self) -> bool {
        self.is_running
    }

    pub fn exit(&mut self) {
        self.is_running = false;
    }
}
#[derive(Default)]
pub struct FileExplorer {
    pub selected: Option<PathBuf>,
}

impl FileExplorer {
    /// Returns true if the given path is selected (meaning we store it as internal state)
    pub fn is_path_selected(&self, p: &Path) -> bool {
        if let Some(ref x) = self.selected {
            x == p
        } else {
            false
        }
    }

    /// Save the selected path
    pub fn select_path(&mut self, p: &Path) {
        self.selected = Some(p.to_owned());
    }

    pub fn reset(&mut self) {
        self.selected = None;
    }
}

fn get_filename_lossy(path: &Path) -> io::Result<Cow<str>> {
    path.file_name()
        .map(|x| x.to_string_lossy())
        .ok_or(io::Error::new(io::ErrorKind::Other, ""))
}

fn visit_dirs(ui: &Ui, file_explorer: &mut FileExplorer, dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let display_name = get_filename_lossy(&path)?;
            if path.is_dir() {
                ui.tree_node(&im_str!("{}", display_name))
                    .default_open(false)
                    .build(|| {
                        if let Err(e) = visit_dirs(ui, file_explorer, &path) {
                            error!("Unexpected error while displaying file explorer = {}", e);
                        }
                    });
            } else {
                let is_selected = file_explorer.is_path_selected(&path);
                if Selectable::new(&im_str!("{}", display_name))
                    .selected(is_selected)
                    .build(ui)
                {
                    file_explorer.select_path(&path);
                }
            }
        }
    }
    Ok(())
}

pub enum UiEvent {
    LoadRom,
    SaveState,
    LoadState,
    ChangeSound,
}

pub fn run_ui(ui: &Ui, application: &mut Application) -> Option<UiEvent> {
    let mut event = None;
    ui.main_menu_bar(|| {
        ui.menu(im_str!("File"), true, || {
            if MenuItem::new(im_str!("Open rom")).build(&ui) {
                application.file_explorer_opened = true;
                application.file_explorer.reset();
            }

            if MenuItem::new(im_str!("Exit")).build(&ui) {
                application.exit();
            }
        });
        ui.menu(im_str!("State"), true, || {
            if MenuItem::new(im_str!("Save state")).build(&ui) {
                event = Some(UiEvent::SaveState);
            }

            if MenuItem::new(im_str!("Load state")).build(&ui) {
                event = Some(UiEvent::LoadState);
            }
        });

        ui.menu(im_str!("Config"), true, || {
            if MenuItem::new(im_str!("Audio")).build(&ui) {
                application.dirty_sound_levels = application.sound_levels;
                application.sound_config_opened = true;
            }
        });
        ui.menu(im_str!("Debug"), false, || {});
    });

    if application.file_explorer_opened {
        Window::new(im_str!("Open ROM"))
            .size([300.0, 300.0], Condition::FirstUseEver)
            .build(&ui, || {
                let current = std::env::current_dir().unwrap();
                if let Err(e) = visit_dirs(ui, &mut application.file_explorer, &current) {
                    error!("Unexpected error while displaying file explorer = {}", e);
                }
                ui.separator();
                if ui.button(im_str!("Load"), [0.0, 0.0]) {
                    event = Some(UiEvent::LoadRom);
                    application.file_explorer_opened = false;
                }
                ui.same_line(0.0);
                if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
                    application.file_explorer_opened = false;
                }
            });
    }

    if application.sound_config_opened {
        Window::new(im_str!("Audio config"))
            .size([600.0, 400.0], Condition::FirstUseEver)
            .build(&ui, || {
                Slider::new(im_str!("Master"), 0..=100)
                    .build(ui, &mut application.dirty_sound_levels.master);
                Slider::new(im_str!("Pulse 1"), 0..=100)
                    .build(ui, &mut application.dirty_sound_levels.pulse_1);
                Slider::new(im_str!("Pulse 2"), 0..=100)
                    .build(ui, &mut application.dirty_sound_levels.pulse_2);
                Slider::new(im_str!("Triangle"), 0..=100)
                    .build(ui, &mut application.dirty_sound_levels.triangle);
                ui.separator();
                if ui.button(im_str!("Ok"), [0.0, 0.0]) {
                    application.sound_levels = application.dirty_sound_levels;
                    event = Some(UiEvent::ChangeSound);
                    application.sound_config_opened = false;
                }
                ui.same_line(0.0);

                if ui.button(im_str!("Apply"), [0.0, 0.0]) {
                    application.sound_levels = application.dirty_sound_levels;
                    event = Some(UiEvent::ChangeSound);
                }
                ui.same_line(0.0);
                if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
                    application.sound_config_opened = false;
                }
            });
    }

    event
}
