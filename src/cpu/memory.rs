use rom;
use ppu::memory::{RegisterType, PpuMemory};
use std::default::Default;

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
    pub ppu_mem: PpuMemory,
}

impl Default for Memory {

    fn default() -> Memory {
        Memory {
            mem: [0; 0x10000],
            ppu_mem: PpuMemory::empty(),
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
        let mut ppu_mem = PpuMemory::new(ines)?;
        Ok(Memory { mem, ppu_mem, ..Default::default()})
    }

    pub fn set(&mut self, address: usize, value: u8) {

        match address {
            0x00..=0x1FFF => self.mem[address & 0x7FFF] = value,
            // These are the PPU registers
            0x2000..=0x2007 => {
                let register_type = RegisterType::lookup(address).unwrap();
                self.ppu_mem.write(register_type, value);
            },
            0x4014 => {
                self.ppu_mem.write_oamdma(&self.mem, value);    
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
            0x2000..=0x2007 => {
                let register_type = RegisterType::lookup(address).unwrap();
                self.ppu_mem.read(register_type)
            },
            0x4014 => self.ppu_mem.read(RegisterType::OAMDMA),
            _ => self.mem[address],
        }
    }

    pub fn nmi(&self) -> bool {
        self.ppu_mem.get_nmi_occured()
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

}

#[cfg(test)]
mod tests {
    use super::*;

    // If vblank occured flag is 1, reading ppustatus will set it to 0.
    #[test]
    fn test_readppustatus_flag_vblank_to_off() {
        let mut memory: Memory = Default::default();
        memory.ppu_mem.update(RegisterType::PPUSTATUS, 0x90);  

        assert_eq!(0x90, memory.get(0x2002));
        assert_eq!(0x10, memory.ppu_mem.peek(RegisterType::PPUSTATUS));
    }


    #[test]
    fn test_set_nmi_status_then_ctrl() {
        let mut memory : Memory = Default::default();
        assert_eq!(false, memory.nmi());
        memory.ppu_mem.update(RegisterType::PPUSTATUS, 0x80);
        assert_eq!(false, memory.nmi());
        memory.set(0x2000, 0x90);
        assert_eq!(true, memory.nmi());
    }

    #[test]
    fn test_set_nmi_ctrl_then_status() {
        let mut memory: Memory = Default::default();
        assert_eq!(false, memory.nmi());
        memory.set(0x2000, 0x90);
        assert_eq!(false, memory.nmi());
        memory.ppu_mem.update(RegisterType::PPUSTATUS, 0x80);
        assert_eq!(true, memory.nmi());
    }

}
