use std::fmt;
use rom;

// Behaviour of PPU register is quite special. For example, when reading PPUSTATUS,
// the vblank flag will be cleared. In order to avoid cluttering the logic in 
// Memory.rs, I'll gather all the ppu register behaviour here.
#[allow(non_snake_case)]
#[derive(Debug)]
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

    // Nametables are used to draw the background. They are basically big
    // 2d arrays. A tile can be 8x8 so the nametable can have 32x30 tiles
    // (256x240 pixels)
    // There is also some mirroring going on but not now.
    vram_addr: u16,
    // when writing to vram_addr, we can only write byte by byte. vram_addr_buffer
    // is here to store the first one.
    vram_addr_buffer: u8,

    // When reading from 0-$3EFF, Place data into buffer and return previous buffer
    // Reading requires a dummy read first. Then you can get the data.
    // This is not the case for palettes, that can be read directly.
    vram_read_buffer: u8,

    // Sprite stuff
    pub oam_addr: u8,
    // object attribute memory. contains the sprite data.
    pub oam: [u8; 0x100],

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
    pub ppu_mem: [u8; 0x4000],
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


    pub fn empty() -> PpuMemory {
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
            vram_addr: 0,
            vram_addr_buffer: 0,
            vram_read_buffer: 0,
            oam_addr: 0,
            oam: [0; 0x100],
            ppu_mem: [0; 0x4000],
        }
    }

    pub fn new(ines: &rom::INesFile) -> Result<PpuMemory, String> {

        // Now the PPU ROM and init
        let mut ppu_mem = [0;0x4000];

        // Just copy the ROM to pattern tables.
        let vrom = ines.get_chr_rom(1)?;
        for (i, b) in vrom.iter().enumerate() {
            ppu_mem[i] = *b;
        }

        Ok(PpuMemory {
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
            vram_addr: 0,
            vram_addr_buffer: 0,
            vram_read_buffer: 0,
            oam_addr: 0,
            oam: [0; 0x100],
            ppu_mem,
        })
    }

    pub fn get_nmi_occured(&self) -> bool {
        self.nmi
    }

    pub fn consume_nmi(&mut self) {
        self.nmi = false;
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
            },
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
    pub fn write(&mut self, register_type: RegisterType, value: u8) {
        match register_type {
            PPUCTRL => self.write_ctrl(value),
            PPUMASK => self.write_mask(value),
            PPUADDR => self.write_addr(value),
            PPUDATA => self.write_data(value),
            OAMADDR => self.write_oamaddr(value),
            OAMDATA => self.write_oamdata(value),
            OAMDMA => panic!("Use directly 'write_oamdma'"), 
            PPUSCROLL => {}, //println!("PPUSCROLL not implemented yet!"),
            _ => panic!("{:?} cannot be written by CPU", register_type),
        }
    }

    /// Read with side-effect
    pub fn read(&mut self, register_type: RegisterType) -> u8 {
        match register_type {
            // Those cannot be read by the CPU
            PPUCTRL | PPUMASK | OAMADDR | PPUSCROLL | PPUADDR | OAMDMA => {
                panic!("{:?} cannot be read by CPU", register_type);
            },
            PPUSTATUS => self.read_status(),
            PPUDATA => self.read_data(),
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
        self.vram_addr = 0;
        old_status
    }

    fn write_ctrl(&mut self, ctrl: u8) {
        self.ppuctrl = ctrl;
        self.raise_nmi();
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
            self.oam[self.oam_addr as usize +i] = *b;
        }
    }

    // Background addr and data
    fn write_addr(&mut self, addr_byte: u8) {
        let old_vram_buf = self.vram_addr_buffer as u16;
        self.vram_addr = (old_vram_buf << 8) + (addr_byte as u16);
        self.vram_addr_buffer = addr_byte;
        self.ppuaddr = addr_byte; // useless?
    }

    fn write_data(&mut self, data: u8) {
        // TODO PALETTE MIRRORS.
        let addr_latch = self.vram_addr;
        self.ppu_mem[(addr_latch as usize) % 0x4000] = data;
        if self.ppuctrl & 4 == 4 {
            self.vram_addr = addr_latch + 32;
        } else {
            self.vram_addr = addr_latch + 1;
        }
    }

    fn read_data(&mut self) -> u8 {
        let addr_latch = self.vram_addr;
        // TODO SHould not do buffering for palettes
        let old_buffer = self.vram_read_buffer;
        self.vram_read_buffer = self.ppu_mem[(addr_latch as usize) % 0x4000];
        if self.ppuctrl & 4 == 4 {
            self.vram_addr = addr_latch + 32;
        } else {
            self.vram_addr = addr_latch + 1;
        }

        old_buffer
    }

    fn raise_nmi(&mut self) {
        self.nmi = (self.ppustatus & 0x80 == 0x80) &&
            (self.ppuctrl & 0x80 == 0x80);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_set_vram_addr() {

        let mut memory = PpuMemory::empty();
        memory.write(RegisterType::PPUADDR, 0x20);
        memory.write(RegisterType::PPUADDR, 0x09);

        assert_eq!(0x2009, memory.vram_addr);
    }
}
