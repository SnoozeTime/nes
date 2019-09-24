use crate::joypad::{InputAction, InputState, Player};

// Emulator specific action
pub enum EmulatorInput {
    PAUSE,
    QUIT,
    DEBUG,
    SAVE,
    INPUT(Player, InputAction, InputState),
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
