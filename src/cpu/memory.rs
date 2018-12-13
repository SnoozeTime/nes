use rom;
use std::default::Default;

// PPU registers
pub mod ppu_register {
    pub const PPUCTRL: usize = 0x2000;
    pub const PPUMASK: usize = 0x2001;
    pub const PPUSTATUS: usize = 0x2002;
    pub const OAMADDR: usize = 0x2003;
    pub const OAMDATA: usize = 0x2004;
    pub const PPUSCROLL: usize = 0x2005;
    pub const PPUADDR: usize = 0x2006;
    pub const PPUDATA: usize = 0x2007;
    pub const OADDMA: usize = 0x4014;
}
// 
// All memory for the NES will be here. It includes CPU ram but also
// PPU ram and all the mapped rom stuff.
//
// It is easier to do it that way because some memory is mapped between
// CPU and PPU ($2000-$2007). Also, write to 2006 and 2007 will write to
// the VRAM. Read from 2007 will read from VRAM.
//
pub struct Memory {

    // Interrupt flag
    pub nmi: bool,

    // memory layout for CPU
    // ---------------
    // Address range    Size    Device
    // $0000-$07FF  $0800   2KB internal RAM
    // $0800-$0FFF  $0800   Mirrors of $0000-$07FF
    // $1000-$17FF  $0800
    // $1800-$1FFF  $0800
    // $2000-$2007  $0008   NES PPU registers
    // $2008-$3FFF  $1FF8   Mirrors of $2000-2007 (repeats every 8 bytes)
    // $4000-$4017  $0018   NES APU and I/O registers
    // $4018-$401F  $0008   APU and I/O functionality that is normally disabled. See CPU Test Mode.
    // $4020-$FFFF  $BFE0   Cartridge space: PRG ROM, PRG RAM, and mapper registers (See Note) 
    pub mem: [u8; 0x10000],    

    // Memory of PPU
    // -------------

    // Pattern tables actually store the tileset used in the game.

    // Nametables are used to draw the background. They are basically big
    // 2d arrays. A tile can be 8x8 so the nametable can have 32x30 tiles
    // (256x240 pixels)
    // There is also some mirroring going on but not now.
    pub vram_addr: u16,
    // when writing to vram_addr, we can only write byte by byte. vram_addr_buffer
    // is here to store the first one.
    pub vram_addr_buffer: u8,



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

impl Default for Memory {

    fn default() -> Memory {
        Memory {
            nmi: false,
            mem: [0; 0x10000],
            vram_addr: 0,
            vram_addr_buffer: 0,
            ppu_mem: [0; 0x4000],
        }
    }

}

impl Memory {

    pub fn new(ines: &rom::INesFile) -> Result<Memory, String> {
        let mut mem = [0; 0x10000];

        // if only one page, mirror it.
        let page_nb = ines.get_prg_rom_pages();

        if page_nb == 1 {
            let page = ines.get_prg_rom(1)?;
            for (i, b) in page.iter().enumerate() {
                mem[0x8000+i] = *b;
                mem[0xC000+i] = *b;
            }
        } else {
            let page = ines.get_prg_rom(1)?;
            for (i, b) in page.iter().enumerate() {
                mem[0x8000+i] = *b;
            }
            let page2 = ines.get_prg_rom(2)?;
            for (i, b) in page2.iter().enumerate() {
                mem[0xC000+i] = *b;
            }
        }

        // Now the PPU ROM and init
        let mut ppu_mem = [0;0x4000];

        // Just copy the ROM to pattern tables.
        let vrom = ines.get_chr_rom(1)?;
        for (i, b) in vrom.iter().enumerate() {
            ppu_mem[i] = *b;
        }

        Ok(Memory { mem, ppu_mem, ..Default::default()})
    }

    pub fn set(&mut self, address: usize, value: u8) {

        match address {
            0x00..=0x1FFF => self.mem[address & 0x7FFF] = value,
            ppu_register::PPUCTRL => self.write_ppuctrl(value),
            ppu_register::PPUSTATUS => {
                panic!("PPUSTATUS is read-only");
            },
            ppu_register::PPUADDR => {
                println!("Write PPUADDR");
                self.write_ppuaddr(value);
            },
            ppu_register::PPUDATA => {

                println!("Write PPUDATA");
                self.mem[address] = value;
            },

            _ => self.mem[address] = value,
        }
    }

    pub fn get(&mut self, address: usize) -> u8 {

        match address {
            0..=0x1FFF => {
                // RAM with mirrors
                self.mem[address & 0x7FFF]
            },
            ppu_register::PPUCTRL => {
                // this is WRITE only so panic! on read
                panic!("PPUCTRL is write only");
            },
            ppu_register::PPUADDR => {
                println!("read PPUADDR");
                self.mem[address]
            },
            ppu_register::PPUDATA => {

                println!("read PPUDATA");
                self.mem[address]
            },


            ppu_register::PPUSTATUS => self.read_ppustatus(),
            _ => self.mem[address],
        }
    }

    // Will read without modifying the value. For example, a read to $2002 is supposed
    // to change a flag. Peek will not. This is used for debugging
    pub fn peek(&self, address: usize) -> u8 {
        if address < 0x2000 {
            self.mem[address & 0x7FF]
        } else {
            self.mem[address]
        }
    }

    // These ppuread_* are function used by the PPU to access its
    // registers. They won't modify anything
    // ppuupdate_* will modify.
    pub fn ppuread_ppumask(&self) -> u8 {
        self.mem[ppu_register::PPUMASK]
    }

    // Read PPUCTRL. This is not a real NES access. It is
    // just an accessor for the PPU to set its state.
    // PPUCTRL is set by the CPU.
    pub fn ppuread_ppuctrl(&self) -> u8 {
        self.mem[ppu_register::PPUCTRL]
    }

    pub fn ppuread_ppustatus(&self) -> u8 {
        self.mem[ppu_register::PPUSTATUS]
    }

    // PPUSTATUS is only readable for the CPU. The API can update
    // its state by using this method instead of the memory.set
    pub fn ppuupdate_ppustatus(&mut self, status: u8) {
        // can raise NMI if ppuctrl has nmi flag set.
        let ctrl = self.mem[ppu_register::PPUCTRL];
        self.raise_nmi(ctrl, status);

        self.mem[ppu_register::PPUSTATUS] = status;
    }


    // Writing to PPUCTRL is done through memory.set.
    fn write_ppuctrl(&mut self, ctrl: u8) {
        // if set NMI flag, an interrupt might be immediately generated if
        // vblank flag of ppustatus is up. Vblank flag is 0x80
        let ppustatus = self.mem[ppu_register::PPUSTATUS];
        self.raise_nmi(ctrl, ppustatus);

        self.mem[ppu_register::PPUCTRL] = ctrl;
    }

    fn write_ppuaddr(&mut self, addr_byte: u8) {
        let old_vram_buf = self.vram_addr_buffer as u16;
        self.vram_addr = (old_vram_buf << 8) + (addr_byte as u16); 
        self.vram_addr_buffer = addr_byte;

        // isn't it useless?
        self.mem[ppu_register::PPUADDR] = addr_byte;
    }

    fn raise_nmi(&mut self, ctrl: u8, status: u8) {
        self.nmi = (status & 0x80 == 0x80) && (ctrl & 0x80 == 0x80);
    }

    // Read ppu status will set vblank occured flag to 0.
    fn read_ppustatus(&mut self) -> u8 {
        let old_value = self.mem[ppu_register::PPUSTATUS];
        if self.mem[ppu_register::PPUSTATUS] & 0x80 != 0 {
            self.mem[ppu_register::PPUSTATUS] = old_value ^ (1 << 7);
        }
        old_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // If vblank occured flag is 1, reading ppustatus will set it to 0.
    #[test]
    fn test_readppustatus_flag_vblank_to_off() {
        let mut memory: Memory = Default::default();
        memory.mem[ppu_register::PPUSTATUS] = 0x90;  

        assert_eq!(0x90, memory.get(ppu_register::PPUSTATUS));
        assert_eq!(0x10, memory.mem[ppu_register::PPUSTATUS]);
    }


    #[test]
    fn test_set_nmi_status_then_ctrl() {

        let mut memory : Memory = Default::default();
        assert_eq!(false, memory.nmi);
        memory.ppuupdate_ppustatus(0x80);
        assert_eq!(false, memory.nmi);
        memory.set(ppu_register::PPUCTRL, 0x90);
        assert_eq!(true, memory.nmi);
    }

    #[test]
    fn test_set_nmi_ctrl_then_status() {
        let mut memory: Memory = Default::default();
        assert_eq!(false, memory.nmi);
        memory.set(ppu_register::PPUCTRL, 0x90);
        assert_eq!(false, memory.nmi);
        memory.ppuupdate_ppustatus(0x80);
        assert_eq!(true, memory.nmi);
    }

    #[test]
    fn test_set_vram_addr() {

        let mut memory: Memory = Default::default();
        memory.set(ppu_register::PPUADDR, 0x20);
        memory.set(ppu_register::PPUADDR, 0x09);

        assert_eq!(0x2009, memory.vram_addr);
    }
}
