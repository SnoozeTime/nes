pub mod memory;
pub mod palette;
use super::cpu::memory::Memory;
use self::memory::RegisterType;

fn reverse_bit(mut in_byte: u8) -> u8 {

    let mut out_byte: u8 = 0;
    let mut rest = 8;

    while in_byte != 0 {

        out_byte <<= 1;
        out_byte |= in_byte & 1;
        in_byte >>= 1;
        rest -= 1;
    }

    out_byte << rest

}
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
    ppu_ctrl: u8,

    // 262 line per frame.
    line: usize,
    // 341 cycle per line.
    cycle: usize,

    display_flag: bool,

    // For background rendering.
    // reset at each frame...
    nt: u8, // nametable byte
    at: u8, // attribute table byte
    low_bg_byte: u8,
    high_bg_byte: u8,

    // 2 16-bits shift registers to display background.
    // Every 8 cycles, the bitmap data for the next sprite is loaded in the upper 8 bits
    high_bg_shift_reg: u16,
    low_bg_shift_reg: u16,

    // For sprites
    secondary_oam: [u8; 32],
    nb_sprites: usize,
    // one 8x1 pixels (slice of tile). 8 slices to make a tile.
    pub virtual_buffer: [TileRowInfo; 0x1e00], 
    pub virtual_sprite_buffer: Vec<SpriteInfo>,
}

impl Ppu {

    pub fn new() -> Ppu {
        Ppu { 
            ppu_ctrl: 0,
            line: 0,
            cycle: 0,
            display_flag: false,
            nt: 0,
            at: 0,
            low_bg_byte: 0,
            high_bg_byte: 0,
            high_bg_shift_reg: 0,
            low_bg_shift_reg: 0,
            secondary_oam: [0; 32],
            nb_sprites: 0,
            virtual_buffer: [TileRowInfo::new(0, 0, 0); 0x1e00],
            virtual_sprite_buffer: Vec::new(),
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

    fn tick(&mut self) {

        // skip x if odd frame
        // increase x.
    }

    fn exec_cycle(&mut self, memory: &mut Memory) {

        let ppu_mask = memory.ppu_mem.peek(RegisterType::PPUMASK);
        let ppu_status = memory.ppu_mem.peek(RegisterType::PPUSTATUS);
        let ppu_ctrl = memory.ppu_mem.peek(RegisterType::PPUCTRL);

        let render_bg = ((ppu_mask >> 3) & 1) == 1;
        let render_sprite = ((ppu_mask >> 4) & 1) == 1;
        let rendering_enabled = render_bg || render_sprite;
        let visible_line = self.line < 240;
        let post_render_line = self.line == 240;
        let pre_render_line = self.line == 261;
        let vbl_line = self.line < 261 && self.line > 240;


        let fetch_cycles = (self.cycle > 0 && self.cycle <= 256) || (self.cycle >= 321); 

        // first, display the pixel at (x,y)


        // fetch the pixel info
        if (visible_line || pre_render_line) && fetch_cycles && rendering_enabled {
            // shift registers 2 bits.
            self.high_bg_shift_reg >>= 2;
            self.low_bg_shift_reg >>= 2;

            match self.cycle % 8 {
                2 => self.fetch_nt(memory),
                4 => self.fetch_attr(memory),
                6 => self.fetch_bmp_low(memory),
                0 => {
                    self.fetch_bmp_high(memory);

                    // fetch high bmp and add to internal shift registers
                    self.load_bitmap();           

                    // Increase horizontal v (coarse X)
                    self.coarse_x_increment(memory);
                },
                _ => {},
            }

            if self.cycle == 256 {
                //  increase vertical v (fine y)
                self.y_increment(memory);
            }

            if self.cycle == 257 {
                // copy horizontal t to horizontal v
                self.copy_horizontal_t(memory);
            }
        }

        // Only during the pre-render line, during a few cycles 
        // the vertical t is copied multiple time to vertical v
        if pre_render_line && rendering_enabled && self.cycle >= 280 && self.cycle <= 304 {
            self.copy_vertical_t(memory);
        }
    }

    fn fetch_nt(&mut self, memory: &Memory) {
        let addr = 0x2000 | (memory.ppu_mem.v & 0x0FFF);
        self.nt = memory.ppu_mem.ppu_mem[addr as usize];
    }

    fn fetch_attr(&mut self, memory: &Memory) {
        // attribute address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
        let v = memory.ppu_mem.v;
        let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        self.attr = memory.ppu_mem.ppu_mem[addr as usize];
    }

    fn fetch_bmp_low(&mut self, memory: &Memory) {
        let pattern_table_addr = 0x1000 *
            ((self.ppu_ctrl >> 4) & 1) as usize;
        let bmp_low = self.tile_low_addr(pattern_table_addr,
                                         self.nt as usize,
                                         self.fine_y(memory) as usize);
        self.low_bg_byte = memory.ppu_mem.ppu_mem[bmp_low];
    }

    fn fetch_bmp_high(&mut self, memory: &Memory) {
        // fetch bitmap high. One byte higher than low addr.
        let pattern_table_addr = 0x1000 *
            ((self.ppu_ctrl >> 4) & 1) as usize;
        let addr = self.tile_low_addr(pattern_table_addr,
                                      self.nt as usize,
                                      self.fine_y(memory) as usize);
        let bmp_high = addr + 8;
        self.high_bg_byte = memory.ppu_mem.ppu_mem[bmp_high];
    }

    fn load_bitmap(&mut self) {
        self.high_bg_shift_reg = (self.high_bg_shift_reg & 0xFF) | (self.high_bg_byte as u16 << 8);
        self.low_bg_shift_reg = self.low_bg_shift_reg & 0xFF | (self.low_bg_byte as u16 << 8);
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
        self.ppu_ctrl = memory.ppu_mem.peek(RegisterType::PPUCTRL);

        // no rendering. just add the cycles.
        // No way we add more than one line at a time in the current code...
        for _ in 0..cycles_to_exec {
            if self.line < 240 {
                memory.ppu_mem.is_rendering = true;
                // Visible lines. BACKGROUND
                if (ppu_mask >> 3) & 1 == 1 {
                    if self.cycle == 0 {
                        // lazy cycle
                    } else if self.cycle > 0 && self.cycle <= 256 {
                        // Draw background
                        self.fetch_background(memory);  
                    } else if self.cycle == 257 {
                        // Reset X
                        let v = memory.ppu_mem.v;
                        memory.ppu_mem.v = (v & !0x1F) + (memory.ppu_mem.t & 0x1F);
                    }  else if self.cycle > 320 && self.cycle <= 336 {
                        // fetch the two tiles for the next line
                        if self.line != 239 {
                            self.fetch_background(memory);
                        }
                    }
                }

            } else if self.line == 240 {
                memory.ppu_mem.is_rendering = false;
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

            } else if self.line == 261 {
                // at line 261, it is the end of vblank. We are also going to fetch the
                // tiles for the first line of the next frame.
                if self.cycle == 1 {
                    memory.ppu_mem.update(RegisterType::PPUSTATUS, ppu_status & !0x80);
                }

                if self.cycle > 278 && self.cycle < 305 {
                    let mut v = memory.ppu_mem.v;
                    v = (memory.ppu_mem.v & !0x3E0) | (memory.ppu_mem.t & 0x3e0);
                    memory.ppu_mem.v = v;
                }
                // prefetch data :D
                if (ppu_mask >> 3) & 1 == 1 {
                    if self.cycle == 0 {
                        // lazy cycle
                    } else if self.cycle > 0 && self.cycle <= 256 {
                        // Draw background
                        self.fetch_background(memory);  
                    } else if self.cycle == 257 {
                        // Reset X
                        let v = memory.ppu_mem.v;
                        memory.ppu_mem.v = (v & !0x1F) + (memory.ppu_mem.t & 0x1F);
                    }  else if self.cycle > 320 && self.cycle <= 336 {
                        // fetch the two tiles for the next line
                        self.fetch_background(memory);
                    }
                }

                // SPRITES
                if (ppu_mask >> 4) & 1 == 1 {
                    self.fetch_sprites(memory);
                }

                if self.cycle == 337 {
                    println!("At the end of VBLANK, X is {}, Y is {} and v {:X}", self.coarse_x(memory),
                    self.coarse_y(memory), memory.ppu_mem.v);
                }
            }

            self.cycle = (self.cycle + 1) % 341;
            if self.cycle == 0 {
                self.line += 1;
            }

            self.line = self.line % 262;
        }

        Ok(())
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
            let nametable = 0x1000 * ((self.ppu_ctrl >> 3) & 1) as usize;

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

                let mut tile_low = memory.ppu_mem.ppu_mem[bmp_low];
                let mut tile_high = memory.ppu_mem.ppu_mem[bmp_high];
                if (attr_byte >> 6) & 1 == 1 {
                    // flip horizontally :D
                    tile_low = reverse_bit(tile_low);
                    tile_high = reverse_bit(tile_high);
                }

                self.virtual_sprite_buffer.push(
                    SpriteInfo{ tile: TileRowInfo::new(
                            tile_low,
                            tile_high,
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
                let nt_idx = (self.coarse_x(memory) as usize) + 32 * (self.coarse_y(memory) as usize);
                // let's assume unique screen, only one nametable so far
                self.nt = memory.ppu_mem.ppu_mem[nt_idx+0x2000];
            },
            4 => {
                // fetch attribute byte (colors)
                let attr_idx = (8 * (self.coarse_y(memory) / 4) + self.coarse_x(memory)/4) as usize;
                self.at = memory.ppu_mem.ppu_mem[attr_idx+0x23C0];
            },
            6 => {
                // fetch bitmap low. Address is held in self.nt
                let pattern_table_addr = 0x1000 *
                    ((self.ppu_ctrl >> 4) & 1) as usize;
                let bmp_low = self.tile_low_addr(pattern_table_addr,
                                                 self.nt as usize,
                                                 self.fine_y(memory) as usize);
                self.low_bg_byte = memory.ppu_mem.ppu_mem[bmp_low];
            },
            // 8th cycle
            0 => {
                // fetch bitmap high. One byte higher than low addr.
                let pattern_table_addr = 0x1000 *
                    ((self.ppu_ctrl >> 4) & 1) as usize;
                let addr = self.tile_low_addr(pattern_table_addr,
                                              self.nt as usize,
                                              self.fine_y(memory) as usize);
                let bmp_high = addr + 8;
                self.high_bg_byte = memory.ppu_mem.ppu_mem[bmp_high];

                if self.cycle > 239 && self.cycle <= 257 {

                } else {
                    println!("X:{} Y:{} fine y:{}", self.coarse_x(memory), self.coarse_y(memory), self.fine_y(memory));
                    let idx =
                        (self.coarse_x(memory) as usize) + 32 * ((self.fine_y(memory) as usize) + (8*self.coarse_y(memory) as usize));
                    self.virtual_buffer[idx] = TileRowInfo::new(self.low_bg_byte,
                                                                self.high_bg_byte,
                                                                self.at);
                }
                // Now we can write to the virtual buffer the 8 bits.
                // modify internal register.
                if self.cycle == 256 {
                    self.y_increment(memory);
                } else {
                    self.coarse_x_increment(memory);
                }

            }
            _ => {
                //nothing, intermediate cycles
            }
        }
    }

    fn fine_y(&self, memory: &Memory) -> u8 {
        ((memory.ppu_mem.v & 0x7000) >> 12) as u8
    }

    fn coarse_y(&self, memory: &Memory) -> u8 {
        ((memory.ppu_mem.v & 0x3E0) >> 5) as u8
    }

    fn coarse_x(&self, memory: &Memory) -> u8 {
        (memory.ppu_mem.v & 0x1F) as u8
    }

    fn nametable(&self, memory: &Memory) -> u8 {
        ((memory.ppu_mem.v & 0xC00) >> 10) as u8
    }

    fn coarse_x_increment(&self, memory: &mut Memory) {
        let mut v = memory.ppu_mem.v;
        if (v & 0x1F) == 31 {
            // at the limit of the screen. We need to switch
            // nametable in that case.
            v &= !0x1F; // X = 0

            // Switch nametable.
            v ^= 0x400;
        } else {
            v += 1;
        }

        memory.ppu_mem.v = v;
    }

    fn y_increment(&self, memory: &mut Memory) {

        // yyy NN YYYYY XXXXX
        let mut v = memory.ppu_mem.v;
        if (v & 0x7000) != 0x7000 {
            // fine y is < 7.
            v += 0x1000;
        } else {
            // reset fine.
            v &= !0x7000;

            let mut y = (v & 0x3e0) >> 5;

            if y == 29 {
                y = 0;
                // switch vertical nametable
                v ^= 0x800;
            } else if y == 31 {
                // y can be set out of bound to read attributes. in that case, wrap to 0
                // without changing the nametable.
                y = 0;
            } else {
                y += 1;
            }

            v = (v & !0x3e0) | (y << 5);
        }

        memory.ppu_mem.v = v;
    }

    fn copy_vertical_t(&self, memory: &mut Memory) {
        let t = memory.ppu_mem.t;
        let v = memory.ppu_mem.v;

        let vert_t = t & 0x3e0;
        memory.ppu_mem.v = (v & !0x3e0) | vert_t;
    }

    fn copy_horizontal_t(&self, memory: &mut Memory) {
        let t = memory.ppu_mem.t;
        let v = memory.ppu_mem.v;

        let horiz_t = t & 0x1f;
        memory.ppu_mem.v = (v & !0x1f) | horiz_t;
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

    #[test]
    fn reverse_byte_test() {

        assert_eq!(0b00010000, reverse_bit(0b00001000));
        assert_eq!(0b11010000, reverse_bit(0b00001011));

    }
}

