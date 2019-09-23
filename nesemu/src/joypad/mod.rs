use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Player {
    One,
    Two,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum InputAction {
    A,
    B,
    SELECT,
    START,
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InputState {
    Pressed,
    Released,
}

// The NES supports several different input devices, including joypads, Zapper (light guns), and
// four-player devices.
// Joypad #1 and #2 are read via $4016 and $4017, respectively.
// The joypads are reset via a strobing-method: writing 1, then 0, to $4016. This address controls
// the strobe on both joypads. See "Expansion ports" for information regarding "half-strobing."
// After a full strobe, the joypad's button status will be returned in a single-bit stream (D0).
// Multiple reads need to be made to read all the information about the controller.
// The standard controller can be read 8 times, once for each button:
//
// 1 = A
// 2 = B
// 3 = SELECT
// 4 = START
// 5 = UP
// 6 = DOWN
// 7 = LEFT
// 8 = RIGHT
//
#[derive(Serialize, Deserialize)]
pub struct Joypad {
    current_index: u8, // between 0 and 7 usually.

    // 0 or 1
    a: u8,
    b: u8,
    select: u8,
    start: u8,
    up: u8,
    down: u8,
    left: u8,
    right: u8,

    // to reset the joypad
    reset_buf: u8,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            current_index: 0,
            a: 0,
            b: 0,
            select: 0,
            start: 0,
            up: 0,
            down: 0,
            left: 0,
            right: 0,
            reset_buf: 0,
        }
    }

    pub fn write(&mut self, value: u8) {
        if self.reset_buf == 1 && value == 0 {
            self.current_index = 0;
        }

        self.reset_buf = value;
    }

    pub fn read(&mut self) -> u8 {
        let return_value = match self.current_index {
            0 => self.a,
            1 => self.b,
            2 => self.select,
            3 => self.start,
            4 => self.up,
            5 => self.down,
            6 => self.left,
            7 => self.right,
            _ => 1,
        };

        self.current_index += 1;
        return_value
    }

    pub fn button_up(&mut self, button: &InputAction) {
        match *button {
            InputAction::A => self.a = 0,
            InputAction::B => self.b = 0,
            InputAction::START => self.start = 0,
            InputAction::SELECT => self.select = 0,
            InputAction::UP => self.up = 0,
            InputAction::DOWN => self.down = 0,
            InputAction::RIGHT => self.right = 0,
            InputAction::LEFT => self.left = 0,
        }
    }

    pub fn button_down(&mut self, button: &InputAction) {
        match *button {
            InputAction::A => self.a = 1,
            InputAction::B => self.b = 1,
            InputAction::START => self.start = 1,
            InputAction::SELECT => self.select = 1,
            InputAction::UP => self.up = 1,
            InputAction::DOWN => self.down = 1,
            InputAction::RIGHT => self.right = 1,
            InputAction::LEFT => self.left = 1,
        }
    }
}
