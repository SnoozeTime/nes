use super::cpu::memory::Memory;
use super::ppu::Ppu;

// Emulator specific action
pub enum EmulatorInput {
    PAUSE,
    QUIT,
    DEBUG,
    SAVE,
}
/// Will drawn and get events from input hardware
pub trait GraphicHandler {
    /// Display a frame
    fn display(&mut self, memory: &Memory, ppu: &mut Ppu);

    /// Return events from joystick/keyboard/whatever
    fn handle_events(&mut self, mem: &mut Memory, is_paused: bool) -> Option<EmulatorInput>;
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn RGB(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
