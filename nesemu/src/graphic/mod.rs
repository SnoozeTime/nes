use crate::joypad::{InputAction, InputState, Player};
use crate::nes::Nes;

// Emulator specific action
pub enum EmulatorInput {
    PAUSE,
    QUIT,
    DEBUG,
    SAVE,
    INPUT(Player, InputAction, InputState),
}
/// Will drawn and get events from input hardware
pub trait GraphicHandler {
    /// Display a frame
    fn display(&mut self, nes: &mut Nes);

    /// Return events from joystick/keyboard/whatever
    fn poll_events(&mut self) -> Vec<EmulatorInput>;
}

pub trait Canvas {
    fn set_color(&mut self, color: Color);
    fn clear_state(&mut self);
    fn show(&mut self);
    // TODO return Result.
    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32);
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
