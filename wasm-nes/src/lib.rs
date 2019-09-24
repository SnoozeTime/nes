mod utils;

use nesemu::graphic::EmulatorInput;
use nesemu::nes::Nes;
use nesemu::rom;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct NesEmulator {
    inner: Nes,
}

#[wasm_bindgen]
pub struct EmuInput(EmulatorInput);

const DKKONG: &'static [u8] = include_bytes!("../../games/dk.nes");

#[wasm_bindgen]
impl NesEmulator {
    pub fn new() -> NesEmulator {
        let ines = rom::from_bytes("DKKONG".to_owned(), Vec::from(DKKONG)).unwrap();
        NesEmulator {
            inner: Nes::new(ines).unwrap(),
        }
    }

    /// If true, the main loop should continue
    pub fn should_run(&self) -> bool {
        self.inner.should_run
    }

    /// Execute one CPU tick and 3 PPU ticks
    pub fn tick(&mut self) {
        self.inner.tick(false).unwrap();
    }

    /// Handle events from javascript
    pub fn handle_event(&mut self, event: EmuInput) {
        self.inner.handle_event(event.0);
    }

    /// A frame is ready
    pub fn should_display(&mut self) -> bool {
        self.inner.should_display()
    }

    pub fn width(&self) -> usize {
        self.inner.width()
    }

    pub fn height(&self) -> usize {
        self.inner.height()
    }
    /// -----------------------------
    /// Some debug functions.
    /// -----------------------------
    pub fn log_cpu(&self) {
        log(&format!("{:?}", self.inner.cpu()));
    }
}

#[wasm_bindgen]
pub fn greet() {
    log("Hello, wasm-nes!");
}
