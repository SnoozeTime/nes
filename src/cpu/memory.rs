use std::fmt;
use rom;

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

}

impl Memory {

    pub fn empty() -> Memory {
        Memory { mem: [0;0x10000] }
    }

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

        Ok(Memory { mem })
    }

    pub fn set(&mut self, address: usize, value: u8) {

        match address {
            0x00..=0x1FFF => self.mem[address & 0x7FFF] = value,
            ppu_register::PPUCTRL => self.write_ppuctrl(value),
            ppu_register::PPUSTATUS => {
                panic!("PPUSTATUS is read-only");
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

    // Read PPUCTRL. This is not a real NES access. It is
    // just an accessor for the PPU to set its state.
    // PPUCTRL is set by the CPU.
    pub fn read_ppuctrl(&self) -> u8 {
        self.mem[ppu_register::PPUCTRL]
    }

    // PPUSTATUS is only readable for the CPU. The API can update
    // its state by using this method instead of the memory.set
    pub fn update_ppustatus(&mut self, status: u8) {
        self.mem[ppu_register::PPUSTATUS] = status;
    }


    // Writing to PPUCTRL is done through memory.set.
    fn write_ppuctrl(&mut self, ctrl: u8) {
        // if set NMI flag, an interrupt might be immediately generated if
        // vblank flag of ppustatus is up.

        self.mem[ppu_register::PPUCTRL] = ctrl;
    }

    // Read ppu status will set vblank occured flag to 0.
    fn read_ppustatus(&mut self) -> u8 {
        let old_value = self.mem[ppu_register::PPUSTATUS];
        self.mem[ppu_register::PPUSTATUS] = old_value ^ (1 << 7);
        old_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // If vblank occured flag is 1, reading ppustatus will set it to 0.
    #[test]
    fn test_readppustatus_flag_vblank_to_off() {
        let mut memory = Memory::empty();
        memory.mem[ppu_register::PPUSTATUS] = 0x90;  

        assert_eq!(0x90, memory.get(ppu_register::PPUSTATUS));
        assert_eq!(0x10, memory.mem[ppu_register::PPUSTATUS]);
    }

}
