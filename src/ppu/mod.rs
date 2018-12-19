pub mod memory;
pub mod palette;
use super::cpu::memory::Memory;
use self::memory::RegisterType;

#[derive(Copy, Clone)]
pub struct TileRowInfo {
    pub low: u8,
    pub high: u8,
    pub attr: u8,
}

impl std::fmt::Debug for TileRowInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:08b}", self.low)
    }
}

impl TileRowInfo {
    pub fn new(low: u8, high: u8, attr: u8) -> TileRowInfo {
        TileRowInfo { low, high, attr }
    }
}

pub struct SpriteInfo {
    pub tile: TileRowInfo,
    pub x: usize,
    pub y: usize,
}

/*
 * Fun times.
 * PPU has internal memory (pattern tables, nametable, attributes and so on)
 * It communicates with CPU through registers. Registers are in the CPU
 * memory (from $2000 to $2007) -> https://wiki.nesdev.com/w/index.php/PPU_registers
 *
 */
#[allow(non_snake_case)]
pub struct Ppu {

    // 262 line per frame.
    line: usize,
    // 341 cycle per line.
    cycle: usize,

    display_flag: bool,

    y: u8, // Fine y scrolling
    X: u8, // Coarse X scrolling
    Y: u8, // Coarse Y scrolling

    // For background rendering.
    // reset at each frame...
    nt: u8, // nametable byte
    at: u8, // attribute table byte
    low_bg_byte: u8,
    high_bg_byte: u8,

    // For sprites
    secondary_oam: [u8; 32],
    nb_sprites: usize,
    // one 8x1 pixels (slice of tile). 8 slices to make a tile.
    pub virtual_buffer: [TileRowInfo; 0x1e00], 
    pub virtual_sprite_buffer: Vec<SpriteInfo>,
}

impl Ppu {

    pub fn new() -> Ppu {
        let line = 0;
        let cycle = 0;
        let display_flag = false;
        let y = 0;
        let X = 3;
        let Y = 0;
        let nt = 0;
        let at = 0;
        let low_bg_byte = 0;
        let high_bg_byte = 0;
        let secondary_oam = [0; 32];
        let nb_sprites = 0;
        let virtual_buffer = [TileRowInfo::new(0,0,0); 0x1e00]; 
        let virtual_sprite_buffer = Vec::new();
        Ppu { line,
        cycle,
        display_flag,
        y,
        X,
        Y,
        nt,
        at,
        low_bg_byte,
        high_bg_byte,
        secondary_oam,
        nb_sprites,
        virtual_buffer,
        virtual_sprite_buffer
        }
    }

    // Do not display too much :D
    pub fn should_display(&mut self) -> bool {
        if self.display_flag {
            self.display_flag = false;
            true
        } else {
            false
        }
    }

    // PPU cycles are a bit more complicated than CPU
    // https://wiki.nesdev.com/w/index.php/PPU_frame_timing
    // https://wiki.nesdev.com/w/index.php/PPU_rendering
    //
    // In this emulator, I chose to run the CPU first, then the PPU. The CPU
    // will return the number of cycles it had executed and the PPU will execute
    // 3 times as many cycles.
    pub fn next(&mut self, cycles_to_exec: u64, memory: &mut Memory) -> Result<(), &'static str> {

        let ppu_mask = memory.ppu_mem.peek(RegisterType::PPUMASK);
        let ppu_status = memory.ppu_mem.peek(RegisterType::PPUSTATUS);

        // no rendering. just add the cycles.
        // No way we add more than one line at a time in the current code...
        for _ in 0..cycles_to_exec {
            if self.line < 240 {
                // Visible lines. BACKGROUND
                if (ppu_mask & 0x2 == 0x2) || (ppu_mask & 0x8 == 0x8) {
                    if self.cycle == 0 {
                        // lazy cycle
                    } else if self.cycle > 0 && self.cycle <= 256 {
                        // Draw background
                        self.fetch_background(memory);  
                    } else if self.cycle == 257 {
                        // Reset X
                        self.X = 0;
                    }  else if self.cycle > 320 && self.cycle <= 336 {
                        // fetch the two tiles for the next line
                        if self.line != 239 {
                            self.fetch_background(memory);
                        }
                    }
                }

                // SPRITES
                self.fetch_sprites(memory);

            } else if self.line == 240 {
                // post render line.
            } else if self.line > 240 && self.line < 261 {
                // inside VBlank :)
                if self.line == 241 && self.cycle == 1 {
                    memory.ppu_mem.update(RegisterType::PPUSTATUS, ppu_status | 0x80);
                    // UI object will display the current frame now that we 
                    // are in vblank
                    self.display_flag = true;
                }

                if self.line == 260 {
                    self.virtual_sprite_buffer.clear();
                }
            } else {
                // at line 261, it is the end of vblank. We are also going to fetch the
                // tiles for the first line of the next frame.
                if self.cycle == 1 {
                    memory.ppu_mem.update(RegisterType::PPUSTATUS, ppu_status & !0x80);
                }

                self.y = 0;
                self.Y = 0;
                // prefetch data :D
                self.fetch_background(memory);

                // sprites as well.
                self.fetch_sprites(memory);
            }

            self.cycle = (self.cycle + 1) % 341;
            if self.cycle == 0 {
                self.line += 1;
            }

            self.line = self.line % 262;
        }

        Ok(())
    }

    // TODO modify for scrolling
    fn increase_x(&mut self) {
        // increase coarse X.
        if self.X < 32 {
            self.X += 1
        } else {
            self.X = 0;
        }
    }

    // TODO modify for scrolling
    fn increase_y(&mut self) {
        // Increase first the fine y (3 bits). If overflow, will increase the coarse scrolling.
        if self.y == 7 {
            self.y = 0;
            if self.Y < 30 {
                self.Y += 1;
            } else {
                self.Y = 0;
            }
        } else {
            self.y += 1;
        }

    }

    fn fetch_sprites(&mut self, memory: &mut Memory) {
        // during 1-64, the secondary OAM is cleared and the primary
        // OAM is scanned. Every sprite that will be in the line will
        // be added to the secondary OAM
        if self.cycle == 1 {    
            // Clear secondary OAM
            self.secondary_oam = [0; 32]; 
            self.nb_sprites = 0;
        } else if self.cycle == 65 {
            // populate secondary OAM
            // Find the sprites that are in range for the next Y.
            let mut addr = memory.ppu_mem.oam_addr as usize;
            
            let mut secondary_oam_addr = 0;
            while addr < 0x100 {

                let sprite_y = memory.ppu_mem.oam[addr] as usize;
                // TODO implement for 16 pixels tall.
                if self.line +1 >= sprite_y && self.line+1 <= sprite_y + 8 {
                    self.secondary_oam[secondary_oam_addr] = memory.ppu_mem.oam[addr];
                    self.secondary_oam[secondary_oam_addr+1] = memory.ppu_mem.oam[addr+1];
                    self.secondary_oam[secondary_oam_addr+2] = memory.ppu_mem.oam[addr+2];
                    self.secondary_oam[secondary_oam_addr+3] = memory.ppu_mem.oam[addr+3];
                    secondary_oam_addr += 4;
                    self.nb_sprites += 1;
                }

                // 4 bytes per sprites.
                addr += 4;

                // if we already have 8 sprites, stop here.
                if secondary_oam_addr == 32 {
                    break;
                }
            }
        } else if self.cycle >= 257 && self.cycle < 320 {
            memory.ppu_mem.oam_addr = 0; 
        } else if self.cycle == 320 {
            let ppu_ctrl = memory.ppu_mem.peek(RegisterType::PPUCTRL);
            let nametable = match (ppu_ctrl >> 3) & 1 {
                0 => 0x0,
                1 => 0x1000,
                _ => panic!("Fix that"),
            };
            // print to some virtual buffer
            for i in 0..self.nb_sprites {
                let secondary_oam_addr = 4*i;
                let y = self.line + 1;
                let x = self.secondary_oam[secondary_oam_addr+3] as usize;
                let tile_y = y - self.secondary_oam[secondary_oam_addr] as usize;

                let tile_byte = self.secondary_oam[secondary_oam_addr+1] as usize;
                let bmp_low = self.tile_low_addr(nametable,
                                                 tile_byte,
                                                 tile_y);
                let bmp_high = bmp_low + 8;
                // see bit 3 of PPUCTRL.
                let attr_byte = self.secondary_oam[secondary_oam_addr+2];
                    //self.X, self.Y, self.y);
                    self.virtual_sprite_buffer.push(
                        SpriteInfo{ tile: TileRowInfo::new(
                                memory.ppu_mem.ppu_mem[bmp_low],
                                memory.ppu_mem.ppu_mem[bmp_high],
                                attr_byte),
                                    x,
                                    y});

            }
        }
    }

    fn fetch_background(&mut self, memory: &mut Memory) {
        // fetch the two tiles for the next line
        match self.cycle % 8 {
            2 => {
                // fetch nametable byte, which is the index of the tile
                let nt_idx = (self.X as usize) + 32 * (self.Y as usize);
                // let's assume unique screen, only one nametable so far
                self.nt = memory.ppu_mem.ppu_mem[nt_idx+0x2000];
            },
            4 => {
                // fetch attribute byte (colors)
                let attr_idx = (8 * (self.Y / 4) + self.X/4) as usize;
                self.at = memory.ppu_mem.ppu_mem[attr_idx+0x23C0];
            },
            6 => {
                // fetch bitmap low. Address is held in self.nt
                let bmp_low = self.tile_low_addr(0x1000,
                                                 self.nt as usize,
                                                 self.y as usize);
                // TODO dynamically choose the pattern table based on register.
                self.low_bg_byte = memory.ppu_mem.ppu_mem[bmp_low];
            },
            // 8th cycle
            0 => {
                // fetch bitmap high. One byte higher than low addr.
                let addr = self.tile_low_addr(0x1000,
                                              self.nt as usize,
                                              self.y as usize);
                let bmp_high = addr + 8;
                self.high_bg_byte = memory.ppu_mem.ppu_mem[bmp_high];

                if self.cycle > 239 && self.cycle <= 257 {

                } else {
                    let idx =
                        (self.X as usize) + 32 * ((self.y as usize) + (8*self.Y as usize));
                    //self.X, self.Y, self.y);
                    self.virtual_buffer[idx] = TileRowInfo::new(self.low_bg_byte,
                                                                self.high_bg_byte,
                                                                self.at);
                }
                // Now we can write to the virtual buffer the 8 bits.
                // modify internal register.
                if self.cycle == 256 {
                    self.increase_y();
                } else {
                    self.increase_x();
                }

            }
            _ => {
                //nothing, intermediate cycles
            }
        }
    }

    fn tile_low_addr(&self, pattern_table: usize, tile_nb: usize, fine_y: usize) -> usize {
        pattern_table + 16 * tile_nb + fine_y
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn increase_x_test() {
        let mut ppu = Ppu::new();
        ppu.increase_x();
        assert_eq!(4, ppu.X);
    }

    #[test]
    fn increase_y_test() {
        let mut ppu = Ppu::new();
        ppu.increase_y();
        assert_eq!(0, ppu.Y);
        assert_eq!(1, ppu.y);
        ppu.increase_y();
        ppu.increase_y();
        ppu.increase_y();
        ppu.increase_y();
        ppu.increase_y();
        ppu.increase_y();
        assert_eq!(0, ppu.Y);
        assert_eq!(7, ppu.y);
        ppu.increase_y();
        assert_eq!(1, ppu.Y);
        assert_eq!(0, ppu.y);
    }

}

