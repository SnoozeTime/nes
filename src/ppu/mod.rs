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
    pub fn next(&mut self, cycles_to_exec: u8) -> Result<(), &'static str> {

        

        for _ in 0..cycles_to_exec {
            println!("ppu cycles");
        }
        
        let before_cycle = self.cycle;
        self.cycle += cycles_to_exec % 341;
        // this is flawed. What if we pass two lines at the same time?
        if before_cycle + cycles_to_exec > 341 {
            self.line += 1;
        }
        

        // Check end of frame.
        if line == 262 {

        }

        Ok(())
    }
}



