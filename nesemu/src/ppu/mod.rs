pub mod memory;
pub mod palette;
use self::memory::RegisterType;
use super::cpu::memory::Memory;

use crate::graphic::Color;
use serde_derive::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize)]
pub struct Ppu {
    nmi_timer: u8,
    debug: bool,
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
    x_bg_attr_shift: u16,
    y_bg_attr_shift: u16,

    odd_frame: bool,
    // For sprites
    secondary_oam: Vec<u8>, //; 32],
    nb_sprites: usize,

    // 8 sprites per line!
    high_sprite_bmp_reg: Vec<u8>, //; 8],
    low_sprite_bmp_reg: Vec<u8>,  //; 8],
    x_position_counters: Vec<u8>, //; 8],
    x_position_offset: Vec<u8>,   //; 8],
    sprite_attributes: Vec<u8>,   //; 8],
    is_active: Vec<bool>,         //; 8],

    #[serde(skip)]
    #[serde(default = "empty_screen")]
    pub pixels: [(u8, u8, u8); 0xf000],

    #[serde(skip)]
    #[serde(default = "empty_screen2")]
    pub pixels2: [u8; 0x2D000],

    #[serde(skip)]
    #[serde(default = "palette::build_default_colors")]
    pub colors: [Color; 64],
}

fn empty_screen() -> [(u8, u8, u8); 0xF000] {
    [(0, 0, 0); 0xF000]
}
fn empty_screen2() -> [u8; 0x2D000] {
    [0; 0x2D000]
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            nmi_timer: 0,
            debug: false,
            line: 0,
            cycle: 0,
            display_flag: false,
            nt: 0,
            at: 0,
            low_bg_byte: 0,
            high_bg_byte: 0,
            high_bg_shift_reg: 0,
            x_bg_attr_shift: 0,
            y_bg_attr_shift: 0,
            low_bg_shift_reg: 0,
            odd_frame: false,
            secondary_oam: vec![0; 32],
            nb_sprites: 0,
            high_sprite_bmp_reg: vec![0; 8],
            low_sprite_bmp_reg: vec![0; 8],
            x_position_counters: vec![0; 8],
            x_position_offset: vec![0; 8],
            is_active: vec![false; 8],
            sprite_attributes: vec![0; 8],

            pixels: [(0, 0, 0); 0xf000],
            pixels2: empty_screen2(),
            colors: palette::build_default_colors(),
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

    fn tick(&mut self, is_rendering: bool) {
        self.cycle += 1;

        if self.cycle == 341 {
            self.line = (self.line + 1) % 262;
            self.odd_frame = !self.odd_frame;
            if self.odd_frame && self.line == 0 && is_rendering {
                self.cycle = 1;
            } else {
                self.cycle = 0;
            }
        }
    }

    fn render_pixel(&mut self, memory: &mut Memory, render_bg: bool, render_sprite: bool) {
        let ppu_mask = memory.ppu_mem.peek(RegisterType::PPUMASK);
        let idx = 256 * self.line + (self.cycle - 1);
        let bg_pixel_v = self.fetch_bg_pixel(&memory);
        let bg_pixel = {
            if ((ppu_mask >> 1) & 1 == 0) && self.cycle <= 8 {
                (0, 0, 0)
            } else {
                if render_bg {
                    let attribute = self.fetch_bg_attr(&memory);
                    let palette =
                        palette::get_bg_palette(attribute, &memory.ppu_mem.palettes, &self.colors);

                    let color = match bg_pixel_v {
                        1 => palette.color1,
                        2 => palette.color2,
                        3 => palette.color3,
                        _ => palette.background,
                    };
                    (color.r, color.g, color.b)
                } else {
                    (0, 0, 0)
                }
            }
        };

        let sprite_pixel_data = self.fetch_sprite_pixel(memory, bg_pixel_v != 0);

        // now, pixel priority :)
        // first sprite has priority if many of them. First sprite pixel is the first
        // one pushed to self.pixels.
        //
        let hide_sprite = ((ppu_mask >> 2) & 1 == 1) && (self.cycle <= 8);
        // First of all, do we render sprites?
        if hide_sprite || sprite_pixel_data == None || !render_sprite {
            self.pixels[idx] = bg_pixel;
        } else if let Some(sprite_pixel) = sprite_pixel_data {
            // if sprite has priority, draw it first.
            let bg_priority = sprite_pixel.3 == 1;

            if bg_priority && bg_pixel_v != 0 {
                self.pixels[idx] = bg_pixel;
            } else {
                self.pixels[idx] = (sprite_pixel.0, sprite_pixel.1, sprite_pixel.2);
            }
        }
    }

    /// Return (r,g,b, priority)
    fn fetch_sprite_pixel(
        &mut self,
        memory: &mut Memory,
        has_bg_pixel: bool,
    ) -> Option<(u8, u8, u8, u8)> {
        let mut pixel_data: Option<(u8, u8, u8, u8)> = None;

        // x between 0 and -7 are active.
        for i in 0..8 {
            let is_active = unsafe { *self.is_active.get_unchecked(i) };
            if is_active {
                let bmp_low = unsafe { *self.low_sprite_bmp_reg.get_unchecked(i) };
                let bmp_high = unsafe { *self.high_sprite_bmp_reg.get_unchecked(i) };
                let attr = unsafe { *self.sprite_attributes.get_unchecked(i) };

                // choose the pixel
                let offset = unsafe { *self.x_position_offset.get_unchecked(i) };
                if offset < 8 {
                    unsafe {
                        *self.x_position_offset.get_unchecked_mut(i) += 1;
                    }
                    if pixel_data == None {
                        let low_bit = (bmp_low >> (7 - offset)) & 1;
                        let high_bit = (bmp_high >> (7 - offset)) & 1;
                        let v = low_bit | (high_bit << 1);

                        if i == 0 {
                            // sprite 0 hit detection.
                            // TODO correct implementation ->
                            // https://wiki.nesdev.com/w/index.php/PPU_OAM#Sprite_zero_hits
                            if has_bg_pixel && v != 0 {
                                self.sprite_0_set(memory);
                            }
                        }

                        let bg_priority = (attr >> 5) & 1;
                        let palette = palette::get_sprite_palette(
                            attr & 0b11,
                            &memory.ppu_mem.palettes,
                            &self.colors,
                        );

                        pixel_data = match v {
                            1 => Some((
                                palette.color1.r,
                                palette.color1.g,
                                palette.color1.b,
                                bg_priority,
                            )),
                            2 => Some((
                                palette.color2.r,
                                palette.color2.g,
                                palette.color2.b,
                                bg_priority,
                            )),
                            3 => Some((
                                palette.color3.r,
                                palette.color3.g,
                                palette.color3.b,
                                bg_priority,
                            )),
                            _ => None,
                        }
                    }
                } else {
                    self.is_active[i] = false;
                }
            }
        }

        pixel_data
    }

    fn fetch_bg_pixel(&self, memory: &Memory) -> u8 {
        let x = memory.ppu_mem.x;
        let low_plane_bit = (self.low_bg_shift_reg >> (15 - x)) & 1;
        let high_plane_bit = (self.high_bg_shift_reg >> (15 - x)) & 1;

        (low_plane_bit | (high_plane_bit << 1)) as u8
    }

    fn fetch_bg_attr(&self, memory: &Memory) -> u8 {
        let x = memory.ppu_mem.x;
        let low_plane_bit = (self.x_bg_attr_shift >> (15 - x)) & 1;
        let high_plane_bit = (self.y_bg_attr_shift >> (15 - x)) & 1;

        (low_plane_bit | (high_plane_bit << 1)) as u8
    }

    fn exec_cycle(&mut self, memory: &mut Memory) {
        let ppu_mask = memory.ppu_mem.peek(RegisterType::PPUMASK);
        let ppu_status = memory.ppu_mem.peek(RegisterType::PPUSTATUS);
        let ppu_ctrl = memory.ppu_mem.peek(RegisterType::PPUCTRL);
        let render_bg = ((ppu_mask >> 3) & 1) == 1;
        let render_sprite = ((ppu_mask >> 4) & 1) == 1;
        let rendering_enabled = render_bg || render_sprite;

        self.tick(rendering_enabled);

        let visible_line = self.line < 240;
        let pre_render_line = self.line == 261;

        let fetch_cycles =
            (self.cycle > 0 && self.cycle <= 256) || (self.cycle >= 321 && self.cycle < 337);
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
        if rendering_enabled {
            if (visible_line || pre_render_line) && fetch_cycles {
                self.high_bg_shift_reg <<= 1;
                self.low_bg_shift_reg <<= 1;
                self.x_bg_attr_shift <<= 1;
                self.y_bg_attr_shift <<= 1;

                match self.cycle % 8 {
                    2 => self.fetch_nt(memory),
                    4 => self.fetch_attr(memory),
                    6 => self.fetch_bmp_low(memory, ppu_ctrl),
                    0 => {
                        self.fetch_bmp_high(memory, ppu_ctrl);

                        // fetch high bmp and add to internal shift registers
                        self.load_bitmap(memory);

                        // Increase horizontal v (coarse X)
                        self.coarse_x_increment(memory);
                    }
                    _ => {}
                }

                if self.cycle == 256 {
                    //  increase vertical v (fine y)
                    self.y_increment(memory);
                }
            }

            if (visible_line || pre_render_line) && self.cycle == 257 {
                self.copy_horizontal_t(memory);
            }

            // Only during the pre-render line, during a few cycles
            // the vertical t is copied multiple time to vertical v
            if pre_render_line && self.cycle >= 280 && self.cycle <= 304 {
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

            if visible_line || pre_render_line {
                if self.cycle == 1 {
                    // Clear secondary OAM
                    for b in &mut self.secondary_oam {
                        *b = 0;
                    }
                    self.nb_sprites = 0;
                } else if self.cycle == 65 {
                    // populate secondary OAM
                    // Find the sprites that are in range for the next Y.
                    let mut addr = memory.ppu_mem.oam_addr as usize;
                    let y_lower_bound = if is_16x8_sprites(ppu_ctrl) { 16 } else { 8 };

                    let mut secondary_oam_addr = 0;
                    while addr < 0x100 {
                        let sprite_y = memory.ppu_mem.oam[addr] as usize;
                        let next_line = (self.line + 1) % 240;
                        if next_line >= sprite_y && next_line < sprite_y + y_lower_bound {
                            self.secondary_oam[secondary_oam_addr] = memory.ppu_mem.oam[addr];
                            self.secondary_oam[secondary_oam_addr + 1] =
                                memory.ppu_mem.oam[addr + 1];
                            self.secondary_oam[secondary_oam_addr + 2] =
                                memory.ppu_mem.oam[addr + 2];
                            self.secondary_oam[secondary_oam_addr + 3] =
                                memory.ppu_mem.oam[addr + 3];
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

            // TODO better way is to put this in  write_vram_at code.
            // This is for MMC3 mapper
            if visible_line || pre_render_line {
                self.count_a12(memory);
            }
        }

        // Vertical blank stuff.
        if self.line == 241 && self.cycle == 1 {
            memory
                .ppu_mem
                .update(RegisterType::PPUSTATUS, ppu_status | 0x80);
            self.display_flag = true;
        }

        if pre_render_line && self.cycle == 1 {
            memory
                .ppu_mem
                .update(RegisterType::PPUSTATUS, ppu_status & !0x80);
            self.sprite_0_clear(memory);
        }
    }

    fn count_a12(&self, memory: &mut Memory) {
        //if is_16x8_sprites(ppu_ctrl) {

        //} else {
        if self.cycle == 260 {
            memory.count_12();
        }
        // else {
        //     if self.cycle == 324 {
        //         memory.count_12();
        //     }
        // }
        //}
    }

    fn fetch_quadrant(&self, memory: &Memory) -> u8 {
        let v = memory.ppu_mem.v();

        ((v >> 1) & 1 | ((v >> 6) & 1) << 1) as u8
    }

    fn fetch_nt(&mut self, memory: &Memory) {
        let addr = 0x2000 | (memory.ppu_mem.v() & 0x0FFF);
        self.nt = memory.read_vram_at(addr as usize);
    }

    fn evaluate_sprites(&mut self, memory: &Memory, ppu_ctrl: u8) {
        //  at this point, the sprites for current line
        //  are already rendered so we can update the registers
        //  for next line.
        let eightb_nametable = 0x1000 * ((ppu_ctrl >> 3) & 1) as usize;
        let is_16b = is_16x8_sprites(ppu_ctrl);
        for i in 0..8 {
            if i <= self.nb_sprites {
                let secondary_oam_addr = 4 * i;
                let y = (self.line + 1) % 240;
                let x = self.secondary_oam[secondary_oam_addr + 3];

                let tile_byte = self.secondary_oam[secondary_oam_addr + 1] as usize;

                let nametable = if is_16b {
                    ((tile_byte & 1) * 0x1000) as usize
                } else {
                    eightb_nametable
                };

                let mut tile_addr = if is_16b { tile_byte & !1 } else { tile_byte };

                let attr_byte = self.secondary_oam[secondary_oam_addr + 2];

                let mut tile_y = y - self.secondary_oam[secondary_oam_addr] as usize;
                let mut bottom_tile = false;
                if tile_y > 7 {
                    tile_y = tile_y % 8;
                    bottom_tile = true;
                }

                if (attr_byte >> 7) & 1 == 1 {
                    // reverse y...
                    //
                    tile_y = 7 - tile_y;
                    bottom_tile = !bottom_tile;
                }

                if bottom_tile && is_16b {
                    tile_addr += 1;
                }

                let bmp_low = self.tile_low_addr(nametable, tile_addr, tile_y);
                let bmp_high = bmp_low + 8;
                // see bit 3 of PPUCTRL.

                let mut tile_low = memory.read_vram_at(bmp_low);
                let mut tile_high = memory.read_vram_at(bmp_high);
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
        let v = memory.ppu_mem.v();
        let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        self.at = memory.read_vram_at(addr as usize);
    }

    fn fetch_bmp_low(&mut self, memory: &Memory, ppu_ctrl: u8) {
        let pattern_table_addr = 0x1000 * ((ppu_ctrl >> 4) & 1) as usize;
        let bmp_low = self.tile_low_addr(
            pattern_table_addr,
            self.nt as usize,
            self.fine_y(memory) as usize,
        );
        self.low_bg_byte = memory.read_vram_at(bmp_low);
    }

    fn fetch_bmp_high(&mut self, memory: &Memory, ppu_ctrl: u8) {
        // fetch bitmap high. One byte higher than low addr.
        let pattern_table_addr = 0x1000 * ((ppu_ctrl >> 4) & 1) as usize;
        let addr = self.tile_low_addr(
            pattern_table_addr,
            self.nt as usize,
            self.fine_y(memory) as usize,
        );
        let bmp_high = addr + 8;
        self.high_bg_byte = memory.read_vram_at(bmp_high);
    }

    fn load_bitmap(&mut self, memory: &Memory) {
        self.high_bg_shift_reg = (self.high_bg_shift_reg & 0xFF00) | (self.high_bg_byte as u16);
        self.low_bg_shift_reg = (self.low_bg_shift_reg & 0xFF00) | (self.low_bg_byte as u16);

        let quadrant = self.fetch_quadrant(memory);
        let attribute = (self.at >> (2 * quadrant)) & 0b11;

        self.x_bg_attr_shift = (self.x_bg_attr_shift & 0xFF00) | (0xFF * (attribute as u16 & 1));
        self.y_bg_attr_shift =
            (self.y_bg_attr_shift & 0xFF00) | (0xFF * ((attribute as u16 >> 1) & 1));
    }

    pub fn next(
        &mut self,
        cycles_to_exec: u64,
        memory: &mut Memory,
        debug: bool,
    ) -> Result<(), &'static str> {
        self.debug = debug;

        for _ in 0..cycles_to_exec {
            self.exec_cycle(memory);
        }

        Ok(())
    }

    fn fine_y(&self, memory: &Memory) -> u8 {
        ((memory.ppu_mem.v() & 0x7000) >> 12) as u8
    }

    fn coarse_x_increment(&self, memory: &mut Memory) {
        let mut v = memory.ppu_mem.v();
        if (v & 0x1F) == 31 {
            // at the limit of the screen. We need to switch
            // nametable in that case.
            v &= !0x1F; // X = 0

            // Switch nametable.
            v ^= 0x400;
        } else {
            v += 1;
        }

        memory.ppu_mem.set_v(v);
    }

    fn y_increment(&self, memory: &mut Memory) {
        // yyy NN YYYYY XXXXX
        let mut v = memory.ppu_mem.v();
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

        memory.ppu_mem.set_v(v);
    }

    fn copy_vertical_t(&self, memory: &mut Memory) {
        let t = memory.ppu_mem.t;
        let v = memory.ppu_mem.v();
        memory.ppu_mem.set_v((v & 0x841F) | (t & 0x7BE0));
    }

    fn copy_horizontal_t(&self, memory: &mut Memory) {
        let t = memory.ppu_mem.t;
        let v = memory.ppu_mem.v();
        memory.ppu_mem.set_v((v & 0xFBE0) | (t & 0x041F));
    }

    fn tile_low_addr(&self, pattern_table: usize, tile_nb: usize, fine_y: usize) -> usize {
        pattern_table + 16 * tile_nb + fine_y
    }

    fn sprite_0_set(&self, memory: &mut Memory) {
        let ppu_status = memory.ppu_mem.peek(RegisterType::PPUSTATUS);
        memory
            .ppu_mem
            .update(RegisterType::PPUSTATUS, ppu_status | 0x40);
    }

    fn sprite_0_clear(&self, memory: &mut Memory) {
        let ppu_status = memory.ppu_mem.peek(RegisterType::PPUSTATUS);
        memory
            .ppu_mem
            .update(RegisterType::PPUSTATUS, ppu_status & !0x40);
    }
}

fn is_16x8_sprites(ppu_ctrl: u8) -> bool {
    (ppu_ctrl >> 5) & 1 == 1
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
