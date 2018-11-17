// Will contain memory layout and access methods
//

// memory layout
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
pub struct Memory {
    mem: [u8; 0xFFFF],    
}

impl Memory {

    pub fn new(rom: Vec<u8>) -> Memory {
        let mut mem: [u8; 0xFFFF] = [0; 0xFFFF];
        // $8000-$FFFF = Usual ROM, commonly with Mapper Registers (see MMC1 and UxROM for example)
        for (i, b) in rom.iter().enumerate() {
            mem[0x8000+i] = *b;
        }

        Memory { mem }
    }

    pub fn set(&mut self, address: usize, value: u8) {
        self.mem[address] = value;
    }

    pub fn get(&self, address: usize) -> u8 {
        self.mem[address]
    }
}
// ----------------------------------------------
// Addressing modes
// ================
// There are different way of accessing data from memory for a same instruction
// For example, LDA can get the value to store in A directly after the instruction
// (addressing mode is immediate). It can also get it from the zero page memory
// and so on.
//
// The instruction enumeration will store a struct that implements AddressingMode.
// The struct will have all the relevant information to fetch from the memory.
// For example, ZeroPageAddressing will store the address of the value to fetch.
//
// This is nice to keep for debugging.
enum AddressingModeType {
    ZERO_PAGE,
    IMMEDIATE,
}

pub trait AddressingMode {
    fn mode_type() -> AddressingModeType where Self: Sized;

    // Will get the value from memory.
    fn fetch(&self, mem: &Memory) -> u8;

    // Debug print
    fn debug(&self) -> String;
}

// Immediate addressing. Just get the value from the next instruction
// -----------------------------------------------------------------

pub struct ImmediateAddressing {
    value: u8,
}

impl ImmediateAddressing {
    pub fn new(value: u8) -> Box<ImmediateAddressing> {
        Box::new(ImmediateAddressing { value })
    }
}

impl AddressingMode for ImmediateAddressing {
    fn mode_type() -> AddressingModeType {
        AddressingModeType::IMMEDIATE
    }

    fn fetch(&self, _mem: &Memory) -> u8 {
        // memory super useless in that case.
        self.value
    }

    fn debug(&self) -> String {
        format!("Immediate adressing: 0x{:x}", self.value)
    }
}

// Zero page addressing. Store the address of the value in the zero-page
// area of memory.
// ---------------------------------------------------------------------

pub struct ZeroPageAddressing {
    address: u8,
}

impl ZeroPageAddressing {
    pub fn new(address: u8) -> Box<ZeroPageAddressing> {
        Box::new(ZeroPageAddressing { address })
    }
}

impl AddressingMode for ZeroPageAddressing {
    fn mode_type() -> AddressingModeType {
        AddressingModeType::ZERO_PAGE
    }

    fn fetch(&self, mem: &Memory) -> u8 {
        // memory super useless in that case.
        mem.get(self.address as usize)
    }

    fn debug(&self) -> String {
        format!("Zero-page adressing at: 0x{:x}", self.address)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_immediate() {
        let mut memory = Memory::new(vec![0;5]);
        let addressing = ImmediateAddressing::new(8);
        assert_eq!(8, addressing.fetch(&memory));
    }

    #[test]
    fn test_zero_page() {
        let mut memory = Memory::new(vec![1, 2 ,3 ,4 ,5]);
        memory.mem[0x02] = 3;
        let addressing = ZeroPageAddressing::new(0x02);
        assert_eq!(3, addressing.fetch(&memory));
    }
}
