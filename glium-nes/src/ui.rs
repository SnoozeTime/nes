use std::borrow::Cow;
use std::default::Default;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tracing::error;

use imgui::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AppState {
    Nothing,
    Running,
    Opening,
}

pub struct Application {
    current_state: AppState,
    previous_state: AppState,
    is_running: bool,
    file_explorer: FileExplorer,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            current_state: AppState::Nothing,
            previous_state: AppState::Nothing,
            is_running: true,
            file_explorer: FileExplorer::default(),
        }
    }
}

impl Application {
    /// Return the rom name to load if selected.
    pub fn rom_name(&self) -> Option<&PathBuf> {
        self.file_explorer.selected.as_ref()
    }

    /// Get the current state of the application
    pub fn current_state(&self) -> AppState {
        self.current_state
    }

    pub fn set_state(&mut self, new_state: AppState) {
        self.current_state = new_state;
    }

    pub fn reset_to_previous(&mut self) {
        self.current_state = self.previous_state;
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
    Resume,
    SaveState,
    LoadState,
}

pub fn run_ui(ui: &Ui, application: &mut Application) -> Option<UiEvent> {
    let mut event = None;
    ui.main_menu_bar(|| {
        ui.menu(im_str!("File"), true, || {
            if MenuItem::new(im_str!("Open rom")).build(&ui) {
                application.file_explorer.reset();
                application.previous_state = application.current_state;
                application.current_state = AppState::Opening;
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
        ui.menu(im_str!("Debug"), false, || {});
    });

    if let AppState::Opening = application.current_state {
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
                }
                ui.same_line(0.0);
                if ui.button(im_str!("Cancel"), [0.0, 0.0]) {
                    event = Some(UiEvent::Resume);
                }
            });
    }

    event
}
