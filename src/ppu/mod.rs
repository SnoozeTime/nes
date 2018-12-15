pub mod memory;
use super::cpu::memory::Memory;
use self::memory::RegisterType;
/*
 * Fun times.
 * PPU has internal memory (pattern tables, nametable, attributes and so on)
 * It communicates with CPU through registers. Registers are in the CPU
 * memory (from $2000 to $2007) -> https://wiki.nesdev.com/w/index.php/PPU_registers
 *
 */

pub struct Ppu {

    // 262 line per frame.
    line: usize,
    // 341 cycle per line.
    cycle: usize,
}

impl Ppu {

    pub fn new() -> Ppu {
        let line = 0;
        let cycle = 0;
        Ppu { line, cycle }
    }

    /// Execute one PPU cycle
    /// There are 3 ppu cycles for each cpu cycle.
    // PPU cycles are a bit more complicated than CPU
    // https://wiki.nesdev.com/w/index.php/PPU_frame_timing
    // https://wiki.nesdev.com/w/index.php/PPU_rendering
    //
    // In this emulator, I chose to run the CPU first, then the PPU. The CPU
    // will return the number of cycles it had executed and the PPU will execute
    // 3 times as many cycles.
    pub fn next(&mut self, cycles_to_exec: u8, memory: &mut Memory) -> Result<(), &'static str> {

        let ppu_mask = memory.ppu_mem.peek(RegisterType::PPUMASK);
        let ppu_status = memory.ppu_mem.peek(RegisterType::PPUSTATUS);

        // no rendering. just add the cycles.
        // No way we add more than one line at a time in the current code...
        for _ in 0..cycles_to_exec {
            if (ppu_mask & 0x2 == 0x2) || (ppu_mask & 0x8 == 0x8) {
                //  println!("Show background");
            }
            self.cycle = (self.cycle + 1) % 341;
            if self.cycle == 0 {
                self.line += 1;
            }

            //
            if self.line == 241 && self.cycle == 1 {
                memory.ppu_mem.update(RegisterType::PPUSTATUS, ppu_status | 0x80);
            }

            if self.line == 261 && self.cycle == 0 {
                memory.ppu_mem.update(RegisterType::PPUSTATUS, ppu_status & !0x80);
            }
        }



        // let before_cycle = self.cycle;
        // self.cycle += cycles_to_exec % 341;
        // // this is flawed. What if we pass two lines at the same time?
        // if before_cycle + cycles_to_exec > 341 {
        //     self.line += 1;
        // }
        // 

        // // Check end of frame.
        // if line == 262 {

        // }

        Ok(())
    }
}



