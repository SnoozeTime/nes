use crate::joypad::{InputAction, InputState, Player};
use serde_derive::{Deserialize, Serialize};

// Emulator specific action
pub enum EmulatorInput {
    PAUSE,
    QUIT,
    DEBUG,
    SAVE,
    INPUT(Player, InputAction, InputState),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}
