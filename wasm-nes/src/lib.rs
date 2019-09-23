mod utils;

use nesemu::nes::Nes;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct NesEmulator {
    inner: Nes,
}

#[wasm_bindgen]
impl NesEmulator {
    pub fn tick(&mut self) {
        self.inner.tick(false).unwrap();
    }
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm-nes!");
}
