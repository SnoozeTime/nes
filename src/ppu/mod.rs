pub mod memory;
pub mod palette;
use super::cpu::memory::Memory;
use self::memory::RegisterType;
use std::collections::HashMap;

use sdl2::pixels::Color;
use serde_derive::{Serialize, Deserialize};

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

/*
 * Fun times.
 * PPU has internal memory (pattern tables, nametable, attributes and so on)
 * It communicates with CPU through registers. Registers are in the CPU
 * memory (from $2000 to $2007) -> https://wiki.nesdev.com/w/index.php/PPU_registers
 *
 */
#[allow(non_snake_case)]
pub struct Ppu {

    debug: bool,
    // 262 line per frame.
    line: usize,
    // 341 cycle per line.
    cycle: usize,

    display_flag: bool,

    // For background rendering.
    // reset at each frame...
    nt: u8, // nametable byte
    next_at: u8, //attribute for next tile
    at: u8, // attribute table byte
    low_bg_byte: u8,
    high_bg_byte: u8,

    // 2 16-bits shift registers to display background.
    // Every 8 cycles, the bitmap data for the next sprite is loaded in the upper 8 bits
    high_bg_shift_reg: u16,
    low_bg_shift_reg: u16,

    odd_frame: bool,
    // For sprites
    secondary_oam: [u8; 32],
    nb_sprites: usize,

    // 8 sprites per line!
    high_sprite_bmp_reg: [u8; 8],
    low_sprite_bmp_reg: [u8; 8],
    x_position_counters: [u8; 8],
    x_position_offset: [u8; 8],
    sprite_attributes: [u8; 8],
    is_active: [bool; 8],

    pub pixels: [(u8, u8, u8); 0xf000],

    //#[serde(skip)]
    //#[serde(default = "palette::build_default_colors")]
    background_colors: HashMap<u8, Color>,
}

impl Ppu {

    pub fn new() -> Ppu {
        Ppu { 
            debug: false,
            line: 0,
            cycle: 0,
            display_flag: false,
            nt: 0,
            at: 0,
            next_at: 0,
            low_bg_byte: 0,
            high_bg_byte: 0,
            high_bg_shift_reg: 0,
            low_bg_shift_reg: 0,
            odd_frame: false,
            secondary_oam: [0; 32],
            nb_sprites: 0,
            high_sprite_bmp_reg: [0; 8],
            low_sprite_bmp_reg: [0; 8],
            x_position_counters: [0; 8],
            x_position_offset: [0; 8],
            is_active: [false; 8],
            sprite_attributes: [0; 8],
            pixels: [(0, 0, 0); 0xf000],
            background_colors: palette::build_default_colors(),
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
        self.cycle += 1;

        if self.cycle == 341 {

            self.line = (self.line + 1) % 262;
            self.odd_frame = !self.odd_frame;
            if self.odd_frame && self.line == 0 {
                self.cycle = 1;
            } else {
                self.cycle = 0;
            }
        }
    }

    fn render_pixel(&mut self, memory: &Memory, render_bg: bool, render_sprite: bool) {

        let idx = 256*self.line + (self.cycle - 1); 
        let bg_pixel = {
            if render_bg {

                let box_row = ((self.line/8)% 4) / 2;
                let box_col = ((self.cycle/8)%4) / 2;
                let attribute = match (box_row, box_col) {
                    (0, 0) => self.at & 0b11,
                    (0, 1) => (self.at & 0b1100) >> 2 ,
                    (1, 0) => (self.at & 0b110000) >> 4,
                    (1, 1) => (self.at & 0b11000000) >> 6,
                    _ => panic!("Not possible"),
                };

                let palette = palette::get_bg_palette(attribute, &memory.ppu_mem.palettes, &self.background_colors).expect("Cannot get palette for background");                   
                

                let color = match self.fetch_bg_pixel() {
                    1 => palette.color1,
                    2 => palette.color2,
                    3 => palette.color3,
                    _ => palette.background,
                };

                (color.r, color.g, color.b)
            } else {
                (0, 0, 0)
            }
        };
        if self.debug {
            println!("RENDER -> L:{} C:{} pixel: {:?}", self.line, self.cycle,bg_pixel);
        }

        let sprite_pixels = {

            let mut pixels: Vec<(u8, u8, u8)> = Vec::new();
            if render_sprite {
                // x between 0 and -7 are active.
                for i in 0..8 {    
                    if self.is_active[i] {

                        let bmp_low = self.low_sprite_bmp_reg[i];
                        let bmp_high = self.high_sprite_bmp_reg[i];
                        let attr = self.sprite_attributes[i];

                        // choose the pixel
                        let offset = self.x_position_offset[i];
                        if offset >= 8 {
                            // nothing to display anymore.
                            self.is_active[i] = false;
                        } else {
                            let low_bit = (bmp_low >> (7 - offset)) & 1;
                            let high_bit = (bmp_high >> (7 - offset)) & 1;
                            let v = low_bit | (high_bit << 1);
                            let palette = palette::get_sprite_palette(attr & 0b11, &memory.ppu_mem.palettes, &self.background_colors)
            .expect("In draw-sprite, cannot get sprite_palette");

                            match v {
                                1 => pixels.push((palette.color1.r,
                                                  palette.color1.g,
                                                  palette.color1.b)),
                                2 => pixels.push((palette.color2.r,
                                                  palette.color2.g,
                                                  palette.color2.b)),
                                3 => pixels.push((palette.color3.r,
                                                  palette.color3.g,
                                                  palette.color3.b)),
                                _ => {},
                            }

                            self.x_position_offset[i] += 1;
                        }

                    }
                }

            }
            pixels
        };

        if sprite_pixels.len() > 0 {
            self.pixels[idx] = sprite_pixels[0]; 
        } else {
            self.pixels[idx] = bg_pixel;
        }
    }

    fn fetch_bg_pixel(&self) -> u8 {
        let low_plane_bit = (self.low_bg_shift_reg >> 15) & 1;
        let high_plane_bit = (self.high_bg_shift_reg >> 15) & 1;

        (low_plane_bit | (high_plane_bit << 1)) as u8
    }

    fn exec_cycle(&mut self, memory: &mut Memory) {

        self.tick();

        let ppu_mask = memory.ppu_mem.peek(RegisterType::PPUMASK);
        let ppu_status = memory.ppu_mem.peek(RegisterType::PPUSTATUS);
        let ppu_ctrl = memory.ppu_mem.peek(RegisterType::PPUCTRL);
        let render_bg = ((ppu_mask >> 3) & 1) == 1;
        let render_sprite = ((ppu_mask >> 4) & 1) == 1;
        let rendering_enabled = render_bg || render_sprite;
        let visible_line = self.line < 240;
        let pre_render_line = self.line == 261;

        let fetch_cycles = (self.cycle > 0 && self.cycle <= 256) ||
            (self.cycle >= 321 && self.cycle < 337); 
        let pixel_cycles = (self.cycle > 0 && self.cycle <= 256) && visible_line;

        if self.line == 241 {
            memory.ppu_mem.is_rendering = false;
        } else if self.line == 0 {
            memory.ppu_mem.is_rendering = true;
        }

        // first, display the pixel at (x,y)
        if visible_line && rendering_enabled && pixel_cycles {
            for i in 0..8 {
                if self.x_position_counters[i] != 0 {
                    self.x_position_counters[i] -= 1;
                    if self.x_position_counters[i] == 0 {
                        self.is_active[i] = true;
                    }
                }
            }
            self.render_pixel(memory, render_bg, render_sprite);
        }

        // fetch the pixel info
        if (visible_line || pre_render_line) && fetch_cycles && rendering_enabled {
            self.high_bg_shift_reg <<= 1;
            self.low_bg_shift_reg <<= 1;

            match self.cycle % 8 {
                2 => self.fetch_nt(memory),
                4 => self.fetch_attr(memory),
                6 => self.fetch_bmp_low(memory, ppu_ctrl),
                0 => {
                    self.fetch_bmp_high(memory, ppu_ctrl);

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
        }

        if rendering_enabled && self.cycle == 257 {
            self.copy_horizontal_t(memory);
        }
        // Only during the pre-render line, during a few cycles 
        // the vertical t is copied multiple time to vertical v
        if pre_render_line && rendering_enabled && self.cycle >= 280 && self.cycle <= 304 {
            self.copy_vertical_t(memory);
        }

        // -----------------------------------------------------------
        // Sprites. During rendering cycles, we just fill 
        // the secondary OAM with the sprites of the next line while
        // the sprites of the current line are printed to the screen
        // ------------------------------------------------------------
        // during 1-64, the secondary OAM is cleared and the primary
        // OAM is scanned. Every sprite that will be in the line will
        // be added to the secondary OAM

        if (visible_line || pre_render_line) && rendering_enabled {
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
                    let next_line = (self.line+1)%240;
                    if next_line >= sprite_y && next_line <= sprite_y + 8 {
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
                self.evaluate_sprites(memory, ppu_ctrl);
            }
        }


        // Vertical blank stuff.
        if self.line == 241 && self.cycle == 1 {
            memory.ppu_mem.update(RegisterType::PPUSTATUS, ppu_status | 0x80);
            self.display_flag = true;
        }

        if self.line == 261 && self.cycle == 1 {
            memory.ppu_mem.update(RegisterType::PPUSTATUS, ppu_status & !0x80);
        }
    }

    fn fetch_nt(&mut self, memory: &Memory) {
        let addr = 0x2000 | (memory.ppu_mem.v & 0x0FFF);
        self.nt = memory.ppu_mem.read_vram_at(addr as usize);
        if self.debug {
            println!("L:{} C:{} addr: {:X} NT:{}", self.line, self.cycle, addr, self.nt);
        }
    }

    fn evaluate_sprites(&mut self, memory: &Memory, ppu_ctrl: u8) {
             //println!("At line {}, will evaluate {} sprites", self.line, self.nb_sprites);
                //  at this point, the sprites for current line
                //  are already rendered so we can update the registers
                //  for next line.
                let nametable = 0x1000 * ((ppu_ctrl >> 3) & 1) as usize;
                for i in 0..8 {
                    if i <= self.nb_sprites {
                        let secondary_oam_addr = 4 * i;
                        let y = (self.line + 1) % 240;
                        let x = self.secondary_oam[secondary_oam_addr+3];
                        let tile_byte = self.secondary_oam[secondary_oam_addr+1] as usize;
                        let attr_byte = self.secondary_oam[secondary_oam_addr+2];

                        let mut tile_y = y - self.secondary_oam[secondary_oam_addr] as usize;
                        if (attr_byte >> 7) & 1 == 1 {
                            tile_y = 7 - tile_y;
                        }

                        let bmp_low = self.tile_low_addr(nametable,
                                                         tile_byte,
                                                         tile_y);
                        let bmp_high = bmp_low + 8;
                        // see bit 3 of PPUCTRL.

                        let mut tile_low = memory.ppu_mem.read_vram_at(bmp_low);
                        let mut tile_high = memory.ppu_mem.read_vram_at(bmp_high);
                        if (attr_byte >> 6) & 1 == 1 {
                            // flip horizontally :D
                            tile_low = reverse_bit(tile_low);
                            tile_high = reverse_bit(tile_high);
                        }

                        self.high_sprite_bmp_reg[i] = tile_high;
                        self.low_sprite_bmp_reg[i] = tile_low;
                        self.x_position_counters[i] = x;
                        self.x_position_offset[i] = 0;
                        self.is_active[i] = false;
                        self.sprite_attributes[i] = attr_byte;
                    } else {
                        self.high_sprite_bmp_reg[i] = 0;
                        self.low_sprite_bmp_reg[i] = 0;
                        self.x_position_counters[i] = 0;
                        self.x_position_offset[i] = 0;
                        self.is_active[i] = false;
                        self.sprite_attributes[i] = 0;
                    }
                }

    }

    fn fetch_attr(&mut self, memory: &Memory) {
        // attribute address = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)
        let v = memory.ppu_mem.v;
        let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        self.next_at = memory.ppu_mem.read_vram_at(addr as usize);
    }

    fn fetch_bmp_low(&mut self, memory: &Memory, ppu_ctrl: u8) {
        let pattern_table_addr = 0x1000 *
            ((ppu_ctrl >> 4) & 1) as usize;
        let bmp_low = self.tile_low_addr(pattern_table_addr,
                                         self.nt as usize,
                                         self.fine_y(memory) as usize);
        self.low_bg_byte = memory.ppu_mem.read_vram_at(bmp_low);
    }

    fn fetch_bmp_high(&mut self, memory: &Memory, ppu_ctrl: u8) {
        // fetch bitmap high. One byte higher than low addr.
        let pattern_table_addr = 0x1000 *
            ((ppu_ctrl >> 4) & 1) as usize;
        let addr = self.tile_low_addr(pattern_table_addr,
                                      self.nt as usize,
                                      self.fine_y(memory) as usize);
        let bmp_high = addr + 8;
        self.high_bg_byte = memory.ppu_mem.read_vram_at(bmp_high);
    }

    fn load_bitmap(&mut self) {
        self.at = self.next_at;
        self.high_bg_shift_reg = (self.high_bg_shift_reg & 0xFF00) | (self.high_bg_byte as u16);
        self.low_bg_shift_reg = (self.low_bg_shift_reg & 0xFF00) | (self.low_bg_byte as u16);
    }

    pub fn next(&mut self, cycles_to_exec: u64, memory: &mut Memory, debug: bool) -> Result<(), &'static str> {
        self.debug = debug;

        for _ in 0..cycles_to_exec {
            self.exec_cycle(memory); 
        }

        Ok(())
    }

    fn fine_y(&self, memory: &Memory) -> u8 {
        ((memory.ppu_mem.v & 0x7000) >> 12) as u8
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
                //      println!("bim at line {} and cycle {}", self.line, self.cycle);
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
        memory.ppu_mem.v = (v & 0x841F) | (t & 0x7BE0)
    }

    fn copy_horizontal_t(&self, memory: &mut Memory) {
        let t = memory.ppu_mem.t;
        let v = memory.ppu_mem.v;
        memory.ppu_mem.v = (v & 0xFBE0) | (t & 0x041F)
    }

    fn tile_low_addr(&self, pattern_table: usize, tile_nb: usize, fine_y: usize) -> usize {
        pattern_table + 16 * tile_nb + fine_y
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn reverse_byte_test() {

        assert_eq!(0b00010000, reverse_bit(0b00001000));
        assert_eq!(0b11010000, reverse_bit(0b00001011));

    }
}

