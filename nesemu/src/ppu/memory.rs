use crate::mapper::{MapperType, Mirroring};
use serde_derive::{Deserialize, Serialize};
use std::fmt;

// Behaviour of PPU register is quite special. For example, when reading PPUSTATUS,
// the vblank flag will be cleared. In order to avoid cluttering the logic in
// Memory.rs, I'll gather all the ppu register behaviour here.
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub enum RegisterType {
    PPUCTRL,
    PPUMASK,
    PPUSTATUS,
    OAMADDR,
    OAMDATA,
    PPUSCROLL,
    PPUADDR,
    PPUDATA,
    OAMDMA,
}

impl RegisterType {
    pub fn lookup(value: usize) -> Option<RegisterType> {
        match value {
            0x2000 => Some(RegisterType::PPUCTRL),
            0x2001 => Some(RegisterType::PPUMASK),
            0x2002 => Some(RegisterType::PPUSTATUS),
            0x2003 => Some(RegisterType::OAMADDR),
            0x2004 => Some(RegisterType::OAMDATA),
            0x2005 => Some(RegisterType::PPUSCROLL),
            0x2006 => Some(RegisterType::PPUADDR),
            0x2007 => Some(RegisterType::PPUDATA),
            0x4014 => Some(RegisterType::OAMDMA),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PpuMemory {
    // Interrupt flag
    pub nmi: bool,

    // Registers
    ppuctrl: u8,
    ppumask: u8,
    ppustatus: u8,
    oamaddr: u8,
    oamdata: u8,
    ppuscroll: u8,
    ppuaddr: u8,
    ppudata: u8,
    oamdma: u8,

    // Pattern tables actually store the tileset used in the game.

    // registers for reading/writing vram and printing to screen.
    // in reality, these are 15 bits registers.
    // During reading and writing, t and v are vram addresses.
    // During rendering, t and v are composed like:
    // yyy NN YYYYY XXXXX
    // yyy: fine Y scroll (between 0 and 7. line of tile)
    // NN: Nametable select. (4 logical nametables)
    // YYYYY: Coarse Y (between 0 and 31)
    // XXXXX: Coarse X (between 0 and 31)

    // Temporary VRAM address, can also be though as the address of the
    // top-left corner of the screen.
    pub t: u16,

    // VRAM address
    v: u16,

    // Fine x scroll. 3 bits
    pub x: u8,

    // First or second write toggle. When writing to 2006 or 2005, we need
    // to know if it is the first write or second.
    // if 0, first write. If 1, second write.
    w: u8,

    // When reading from 0-$3EFF, Place data into buffer and return previous buffer
    // Reading requires a dummy read first. Then you can get the data.
    // This is not the case for palettes, that can be read directly.
    vram_read_buffer: u8,

    // Sprite stuff
    pub oam_addr: u8,
    // object attribute memory. contains the sprite data.
    pub oam: Vec<u8>, //; 0x100],

    // Memory layout for  PPU
    // ----------------------
    // $0000-$0FFF  $1000   Pattern table 0
    // $1000-$1FFF  $1000   Pattern Table 1
    // $2000-$23FF  $0400   Nametable 0
    // $2400-$27FF  $0400   Nametable 1
    // $2800-$2BFF  $0400   Nametable 2
    // $2C00-$2FFF  $0400   Nametable 3
    // $3000-$3EFF  $0F00   Mirrors of $2000-$2EFF
    // $3F00-$3F1F  $0020   Palette RAM indexes
    // $3F20-$3FFF  $00E0   Mirrors of $3F00-$3F1F
    pub nametable_1: Vec<u8>, // 0x0400
    pub nametable_2: Vec<u8>, // 0x0400
    pub palettes: Vec<u8>,    //0x0020

    pub is_rendering: bool,
}

impl fmt::Debug for PpuMemory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PPUCTRL:{:X} PPUMASK:{:X} PPUSTATUS:{:X} OAMADDR:{:X} OAMDATA:{:X} PPUSCROLL:{:X} PPUADDR:{:X} PPUDATA:{:X} OAMDMA:{:X}",
               self.ppuctrl,
               self.ppumask,
               self.ppustatus,
               self.oamaddr,
               self.oamdata,
               self.ppuscroll,
               self.ppuaddr,
               self.ppudata,
               self.oamdma)
    }
}

use self::RegisterType::*;
impl PpuMemory {
    pub fn new() -> PpuMemory {
        PpuMemory {
            nmi: false,
            ppuctrl: 0,
            ppumask: 0,
            ppustatus: 0,
            oamaddr: 0,
            oamdata: 0,
            ppuscroll: 0,
            ppuaddr: 0,
            ppudata: 0,
            oamdma: 0,
            t: 0,
            v: 0,
            x: 0,
            w: 0,
            vram_read_buffer: 0,
            oam_addr: 0,
            oam: vec![0; 0x100],
            nametable_1: vec![0; 0x400],
            nametable_2: vec![0; 0x400],
            palettes: vec![0; 0x20],
            is_rendering: false,
        }
    }

    pub fn get_nmi_occured(&self) -> bool {
        self.nmi
    }

    pub fn consume_nmi(&mut self) {
        self.nmi = false;
    }

    pub fn v(&self) -> u16 {
        self.v
    }

    pub fn set_v(&mut self, v: u16) {
        self.v = v;
    }

    /// Peek will return the register value without impacting anything.
    /// Read-only
    pub fn peek(&self, register_type: RegisterType) -> u8 {
        match register_type {
            PPUCTRL => self.ppuctrl,
            PPUMASK => self.ppumask,
            PPUSTATUS => self.ppustatus,
            OAMADDR => self.oamaddr,
            OAMDATA => self.oamdata,
            PPUSCROLL => self.ppuscroll,
            PPUADDR => self.ppuaddr,
            PPUDATA => self.ppudata,
            OAMDMA => self.oamdma,
        }
    }

    /// Update only ONE register. No side effect on others. For example,
    /// the ppu status is set by PPU with hardware (Vblank and so on?)
    pub fn update(&mut self, register_type: RegisterType, value: u8) {
        match register_type {
            PPUCTRL => self.ppuctrl = value,
            PPUMASK => self.ppumask = value,
            PPUSTATUS => {
                self.ppustatus = value;
                self.raise_nmi();
            }
            OAMADDR => self.oamaddr = value,
            OAMDATA => self.oamdata = value,
            PPUSCROLL => self.ppuscroll = value,
            PPUADDR => self.ppuaddr = value,
            PPUDATA => self.ppudata = value,
            OAMDMA => self.oamdma = value,
        }
    }

    /// Write will set new value to register. This can have side effect on
    /// other registers.
    pub fn write(&mut self, register_type: RegisterType, value: u8, mapper: &mut MapperType) {
        match register_type {
            PPUCTRL => self.write_ctrl(value),
            PPUMASK => self.write_mask(value),
            PPUADDR => self.write_addr(value),
            PPUDATA => self.write_data(value, mapper),
            OAMADDR => self.write_oamaddr(value),
            OAMDATA => self.write_oamdata(value),
            OAMDMA => panic!("Use directly 'write_oamdma'"),
            PPUSCROLL => self.write_scroll(value),
            PPUSTATUS => {}
        }
    }

    /// Read with side-effect
    pub fn read(&mut self, register_type: RegisterType, mapper: &MapperType) -> u8 {
        match register_type {
            // Those cannot be read by the CPU
            PPUCTRL | PPUMASK | OAMADDR | PPUSCROLL | PPUADDR | OAMDMA => {
                //           panic!("{:?} cannot be read by CPU", register_type);
                //
                0
            }
            PPUSTATUS => self.read_status(),
            PPUDATA => self.read_data(mapper),
            _ => 8,
        }
    }

    // --------------------------------------------------------------
    // Access the registers.
    // --------------------------------------------------------------
    fn read_status(&mut self) -> u8 {
        let old_status = self.ppustatus;
        self.ppustatus = old_status & !0x80;
        // Reading PPU STATUS Will also clear the address latch.
        self.w = 0;
        // self.t = 0;
        // self.v = 0;
        old_status
    }

    fn write_ctrl(&mut self, ctrl: u8) {
        let nmi_enabled = self.ppuctrl & 0x80 == 0x80;
        self.ppuctrl = ctrl;
        self.t = (self.t & !0xc00) | ((ctrl & 0b11) as u16) << 10;
        if !nmi_enabled {
            self.raise_nmi();
        }
    }

    fn write_mask(&mut self, mask: u8) {
        self.ppumask = mask;
    }

    fn write_oamaddr(&mut self, oamaddr: u8) {
        self.oam_addr = oamaddr;
    }

    fn write_oamdata(&mut self, oamdata: u8) {
        // TODO ignored during rendering.
        // need to add flag is_rendering...
        self.oam[self.oam_addr as usize] = oamdata;
        self.oam_addr += 1;
    }

    pub fn write_oamdma(&mut self, cpu_mem: &[u8], data_addr: u8) {
        let start_range = (data_addr as usize) << 8;
        let end_range = ((data_addr as usize) << 8) + 0xFF; // inclusive.

        // that can overflow and panic hard...
        for (i, b) in cpu_mem[start_range..=end_range].iter().enumerate() {
            self.oam[self.oam_addr as usize + i] = *b;
        }
    }

    fn write_scroll(&mut self, value: u8) {
        /*
        *$2005 first write (w is 0)
        t: ....... ...HGFED = d: HGFED...
        x:              CBA = d: .....CBA
        w:                  = 1

        $2005 second write (w is 1)
        t: CBA..HG FED..... = d: HGFEDCBA
        w:                  = 0

        * */
        if self.w == 0 {
            self.t = ((value >> 3) as u16) | (self.t & !0b11111);
            self.x = value & 0b111;
            self.w = 1;
        } else {
            let masked_t = self.t & !0x73E0;
            let y_value = (value >> 3) as u16;
            let fine_y_value = (value & 0b111) as u16;

            self.t = masked_t | (y_value << 5) | (fine_y_value << 12);
            self.w = 0;
        }
    }

    // Background addr and data
    fn write_addr(&mut self, addr_byte: u8) {
        if self.w == 0 {
            // first write
            self.t = (((addr_byte & 0b111111) as u16) << 8) + (self.t & 0xFF);
            // 14th bit set to 0!.
            self.t &= !(1 << 14);
            self.w = 1;
        } else {
            // second write
            self.t = (self.t & (0xFF00)) | addr_byte as u16;
            self.v = self.t;
            self.w = 0;
        }
    }

    fn write_data(&mut self, data: u8, mapper: &mut MapperType) {
        let addr_latch = self.v;

        self.write_vram_at((addr_latch as usize) % 0x4000, data, mapper);
        if self.ppuctrl & 4 == 4 {
            self.v = addr_latch + 32;
        } else {
            self.v = addr_latch + 1;
        }
    }

    fn write_vram_at(&mut self, addr: usize, data: u8, mapper: &mut MapperType) {
        match addr {
            0x0000..=0x1FFF => {
                mapper.write_chr(addr, data);
            }
            0x2000..=0x23FF => {
                self.write_to_1st_nametable(addr, data);
            }
            0x2400..=0x27FF => match mapper.get_mirroring() {
                Mirroring::HORIZONTAL => self.write_to_1st_nametable(addr, data),
                Mirroring::VERTICAL => self.write_to_2nd_nametable(addr, data),
                Mirroring::ONE_SCREEN => self.write_to_1st_nametable(addr, data),
            },
            0x2800..=0x2BFF => match mapper.get_mirroring() {
                Mirroring::HORIZONTAL => self.write_to_2nd_nametable(addr, data),
                Mirroring::VERTICAL => self.write_to_1st_nametable(addr, data),
                Mirroring::ONE_SCREEN => self.write_to_1st_nametable(addr, data),
            },
            0x2C00..=0x2FFF => {
                if mapper.get_mirroring() == Mirroring::ONE_SCREEN {
                    self.write_to_1st_nametable(addr, data);
                } else {
                    self.write_to_2nd_nametable(addr, data);
                }
            }
            0x3000..=0x3EFF => {
                // Mirrors of 0x2000, 0x2EFFF
                let newaddr = 0x2000 | (addr & 0xFFF);
                self.write_vram_at(newaddr, data, mapper);
            }
            // palettes mirrors
            0x3F00..=0x3FFF => {
                let offset = (addr & 0xFF) % 0x20;
                self.write_palette(offset, data);
            }
            _ => panic!("write_vram_at Out of bounds: {}", addr),
        }
    }

    fn write_palette(&mut self, offset: usize, data: u8) {
        if offset == 0x10 || offset == 0x00 {
            self.palettes[0x00] = data;
            self.palettes[0x10] = data;
        } else {
            self.palettes[offset] = data;
        }
    }

    fn write_to_1st_nametable(&mut self, addr: usize, data: u8) {
        let offset = addr % 0x400;
        self.nametable_1[offset] = data;
    }

    fn write_to_2nd_nametable(&mut self, addr: usize, data: u8) {
        let offset = addr % 0x400;
        self.nametable_2[offset] = data;
    }

    fn read_data(&mut self, mapper: &MapperType) -> u8 {
        let addr_latch = self.v % 0x4000;

        let v = match addr_latch {
            0x3F00..=0x4000 => {
                self.vram_read_buffer = self.read_vram_at(addr_latch as usize, mapper);
                self.vram_read_buffer
            }
            _ => {
                let old_buffer = self.vram_read_buffer;
                self.vram_read_buffer = self.read_vram_at(addr_latch as usize, mapper);
                old_buffer
            }
        };

        if self.ppuctrl & 4 == 4 {
            self.v = addr_latch + 32;
        } else {
            self.v = addr_latch + 1;
        }

        v
    }

    pub fn read_vram_at(&self, addr: usize, mapper: &MapperType) -> u8 {
        match addr {
            0x0..=0x1FFF => mapper.read_chr(addr),
            0x2000..=0x23FF => self.read_from_1st_nametable(addr),
            0x2400..=0x27FF => match mapper.get_mirroring() {
                Mirroring::HORIZONTAL => self.read_from_1st_nametable(addr),
                Mirroring::VERTICAL => self.read_from_2nd_nametable(addr),
                Mirroring::ONE_SCREEN => self.read_from_1st_nametable(addr),
            },
            0x2800..=0x2BFF => match mapper.get_mirroring() {
                Mirroring::HORIZONTAL => self.read_from_2nd_nametable(addr),
                Mirroring::VERTICAL => self.read_from_1st_nametable(addr),
                Mirroring::ONE_SCREEN => self.read_from_1st_nametable(addr),
            },
            0x2C00..=0x2FFF => {
                if mapper.get_mirroring() == Mirroring::ONE_SCREEN {
                    self.read_from_1st_nametable(addr)
                } else {
                    self.read_from_2nd_nametable(addr)
                }
            }
            // Mirrors of 0x2000 - 0x2EFFF
            //            0x3000..=0x33FF => self.read_from_1st_nametable(addr),
            //            0x3400..=0x37FF => {
            //                match mapper.get_mirroring() {
            //                    Mirroring::HORIZONTAL => self.read_from_1st_nametable(addr),
            //                    Mirroring::VERTICAL => self.read_from_2nd_nametable(addr),
            //                    Mirroring::ONE_SCREEN => self.read_from_1st_nametable(addr),
            //                }
            //            },
            //            0x3800..=0x3BFF => {
            //                match mapper.get_mirroring() {
            //                    Mirroring::HORIZONTAL => self.read_from_2nd_nametable(addr),
            //                    Mirroring::VERTICAL => self.read_from_1st_nametable(addr),
            //                    Mirroring::ONE_SCREEN => self.read_from_1st_nametable(addr),
            //                }
            //            },
            //            0x3C00..=0x3EFF => {
            //                if mapper.get_mirroring() == Mirroring::ONE_SCREEN {
            //                    self.read_from_1st_nametable(addr)
            //                } else {
            //                    self.read_from_2nd_nametable(addr)
            //                }
            //            },
            0x3000..=0x3EFF => {
                // Mirrors of 0x2000, 0x2EFFF
                let newaddr = 0x2000 | (addr & 0xFFF);
                self.read_vram_at(newaddr, mapper)
            }
            // palettes
            0x3F00..=0x3FFF => {
                let offset = (addr & 0xFF) % 0x20;
                self.palettes[offset]
            }
            _ => panic!("read_vram_at Out of bounds: {:X}", addr),
        }
    }

    fn read_from_1st_nametable(&self, addr: usize) -> u8 {
        let offset = addr % 0x400;
        self.nametable_1[offset]
    }

    fn read_from_2nd_nametable(&self, addr: usize) -> u8 {
        let offset = addr % 0x400;
        self.nametable_2[offset]
    }

    fn raise_nmi(&mut self) {
        self.nmi = (self.ppustatus & 0x80 == 0x80) && (self.ppuctrl & 0x80 == 0x80);
    }

    // ---------------------------------------------------
    // Virtual nametable. There is space for only 2 nametables
    // in NES vram, but with mirroring the logical tables are 4.
    // ------------------------------------------------------
    pub fn get_logical_table(&self, table_nb: u8, mapper: &MapperType) -> &[u8] {
        match table_nb {
            0 => &self.nametable_1,
            1 => match mapper.get_mirroring() {
                Mirroring::HORIZONTAL => &self.nametable_1,
                Mirroring::VERTICAL => &self.nametable_2,
                Mirroring::ONE_SCREEN => &self.nametable_1,
            },
            2 => match mapper.get_mirroring() {
                Mirroring::VERTICAL => &self.nametable_1,
                Mirroring::HORIZONTAL => &self.nametable_2,
                Mirroring::ONE_SCREEN => &self.nametable_1,
            },
            3 => {
                if mapper.get_mirroring() == Mirroring::ONE_SCREEN {
                    &self.nametable_1
                } else {
                    &self.nametable_2
                }
            }
            _ => panic!("Only 4 nametables"),
        }
    }
}
