use super::cpu::Nes;
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
pub enum AddressingModeType {
    Implied,
    ZeroPage,
    Immediate,
    Relative,
    ZeroPageX,
    ZeroPageY,
    IndexedZeroPage,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndexedAbsolute,
    Indirect,
    PreIndexedIndirect,
    PostIndexedIndirect,
    Accumulator,
}

pub fn create_addressing(addressing_type: AddressingModeType,
                         nes: &mut Nes) -> Box<AddressingMode> {
    use self::AddressingModeType::*;
    match addressing_type {
    Accumulator => AccumulatorAddressing::new(&nes),
    Implied => ImpliedAddressing::new(),
    Immediate => ImmediateAddressing::new(nes.advance()),
    ZeroPage => ZeroPageAddressing::new(nes.advance()),
    ZeroPageX => IndexedZeroPageAddressing::new(nes.advance(), nes.X()),
    ZeroPageY => IndexedZeroPageAddressing::new(nes.advance(), nes.Y()),
    Relative => RelativeAddressing::new(nes.advance()),
    Absolute => {
        let op1 = nes.advance();
        let op2 = nes.advance();
        AbsoluteAddressing::new(op1, op2)
    },
    AbsoluteX => {
        let op1 = nes.advance();
        let op2 = nes.advance();
        IndexedAbsoluteAddressing::new(op1, op2, nes.X())
    },
    AbsoluteY => {
        let op1 = nes.advance();
        let op2 = nes.advance();
        IndexedAbsoluteAddressing::new(op1, op2, nes.Y())
    },
    PreIndexedIndirect => {
        let op = nes.advance();
        PreIndexedIndirectAddressing::new(op, nes.X())
    },
    PostIndexedIndirect => {
        let op = nes.advance();
        PostIndexedIndirectAddressing::new(op, nes.Y())
    },
    _ => panic!("not implemented"),
    }
}

pub trait AddressingMode {
    fn mode_type(&self) -> AddressingModeType;

    // Will get the value from memory.
    fn fetch(&self, mem: &Memory) -> u8;

    // will set the value to memory
    fn set(&self, mem: &mut Memory, value: u8);

    fn debug(&self) -> String;
}

// Implied. Nothinig to fetch. All the instruction is implied by opcode
// --------------------------------------------------------------------
pub struct ImpliedAddressing {}
impl ImpliedAddressing {
    pub fn new() -> Box<ImpliedAddressing> {
        Box::new(ImpliedAddressing{})
    }
}

impl AddressingMode for ImpliedAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Implied
    }

    fn fetch(&self, _mem: &Memory) -> u8 {
        0
    }

    fn set(&self, _mem: &mut Memory, _v: u8) {}

    fn debug(&self) -> String {
        format!("Implied Addressing")
    }
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
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Immediate
    }

    fn fetch(&self, _mem: &Memory) -> u8 {
        // memory super useless in that case.
        self.value
    }

    fn set(&self, _mem: &mut Memory, _v: u8) {}

    fn debug(&self) -> String {
        format!("Immediate adressing: 0x{:x}", self.value)
    }
}

// Relative addressing
// -----------------------------------
pub struct RelativeAddressing {
    offset: u8,
}

impl RelativeAddressing {
    pub fn new(offset: u8) -> Box<RelativeAddressing> {
        Box::new(RelativeAddressing { offset })
    }
}

impl AddressingMode for RelativeAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Relative
    }

    fn fetch(&self, _mem: &Memory) -> u8 {
        self.offset
    }

    fn set(&self, _mem: &mut Memory, _v: u8) {}

    fn debug(&self) -> String {
        format!("Relative adressing: 0x{:x}", self.offset)
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
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::ZeroPage
    }

    fn fetch(&self, mem: &Memory) -> u8 {
        mem.get(self.address as usize)
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        mem.set(self.address as usize, v);
    }

    fn debug(&self) -> String {
        format!("Zero-page adressing at: 0x{:x}", self.address)
    }
}

// Indexed zero page. The address + offset are used to fetch the memory
//---------------------------------------------------------------------
//
pub struct IndexedZeroPageAddressing {
    address: u8,
    offset: u8, // value of a register
}

impl IndexedZeroPageAddressing {
    pub fn new(address: u8, offset: u8) -> Box<IndexedZeroPageAddressing> {
        Box::new(IndexedZeroPageAddressing { address, offset })
    }
}

impl AddressingMode for IndexedZeroPageAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::IndexedZeroPage
    }

    fn fetch(&self, mem: &Memory) -> u8 {
        // Address + offset should always be in the zero-page area. So 0x00FF + 0x0001
        // should be 0x0000 and not 0x0100. This is done here by keeping address and offset
        // as u8.
        mem.get(self.address.wrapping_add(self.offset) as usize)
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        mem.set(self.address.wrapping_add(self.offset) as usize, v);
    }

    fn debug(&self) -> String {
        format!("Indexed Zero-page adressing at: 0x{:x} + 0x{:x}",
                self.address,
                self.offset)
    }
}

// Absolute addressing mode. In  absolute  addressing,  the  address  of  the  data  to  operate on  is  specified  by  the  two  
// operands supplied, least significant byte first
// ----------------------------------------------------------------
pub struct AbsoluteAddressing {
    address: u16, // Create in new function
}

impl AbsoluteAddressing {
    pub fn new(lsb: u8, msb: u8) -> Box<AbsoluteAddressing> {
        let address = ((msb as u16) << 8) + (lsb as u16);
        Box::new(AbsoluteAddressing{ address })
    }
}

impl AddressingMode for AbsoluteAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Absolute
    }

    fn fetch(&self, mem: &Memory) -> u8 {
        mem.get(self.address as usize)
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        mem.set(self.address as usize, v);
    }
    
    fn debug(&self) -> String {
        format!("Absolute adressing at: 0x{:x}", self.address)
    }

}

// Indexed absolute - Same as absolute but with offset
// ----------------------------------------------------

pub struct IndexedAbsoluteAddressing {
    address: u16,
    offset: u8,
}

impl IndexedAbsoluteAddressing {
    pub fn new(lsb: u8, msb: u8, offset: u8) -> Box<IndexedAbsoluteAddressing> {
        let address = ((msb as u16) << 8) + (lsb as u16);
        Box::new(IndexedAbsoluteAddressing{ address, offset })
    }
}

impl AddressingMode for IndexedAbsoluteAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::IndexedAbsolute
    }

    fn fetch(&self, mem: &Memory) -> u8 {
        mem.get((self.address as usize) + (self.offset as usize))
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        mem.set((self.address as usize) + (self.offset as usize), v)
    }
    
    fn debug(&self) -> String {
        format!("Indexed Absolute adressing at: 0x{:x}+0x{:x}",
                self.address,
                self.offset)
    }

}

// Indirect addressing - meh
// Indirect  addressing  takes  two  operands,  forming  a  16-bit  address,  which  identifies  the least significant byte of another address which is where the data can be found. For example if the operands are bb and cc, and ccbb contains xx and ccbb + 1 contains yy, then the real target address is yyxx. 
pub struct IndirectAddressing {
    lsb_location: u16,
}

impl IndirectAddressing {
    pub fn new(lsb: u8, msb: u8) -> Box<IndirectAddressing> {
        let lsb_location = ((msb as u16) << 8) + (lsb as u16);
        Box::new(IndirectAddressing{ lsb_location })
    }
}

impl AddressingMode for IndirectAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Indirect
    }

    fn fetch(&self, mem: &Memory) -> u8 {
        let lsb = mem.get(self.lsb_location as usize);
        let msb = mem.get((self.lsb_location+1) as usize);

        let address = ((msb as u16) << 8) + (lsb as u16);
        mem.get(address as usize)
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        let lsb = mem.get(self.lsb_location as usize);
        let msb = mem.get((self.lsb_location+1) as usize);

        let address = ((msb as u16) << 8) + (lsb as u16);
        mem.set(address as usize, v);
    }

    fn debug(&self) -> String {
        format!("Indirect adressing at: 0x{:x}",
                self.lsb_location)
    }
}


// Indexed indirect (aka pre-indexed)... wtf.
// E.g. LDA ($44, X)
// --------------------------------------------
pub struct PreIndexedIndirectAddressing {
    address: u16, // address is u16 but is always 0x00XX
    offset: u8,
}

impl PreIndexedIndirectAddressing {
    pub fn new(address_byte: u8, offset: u8) -> Box<PreIndexedIndirectAddressing> {
        let address = address_byte as u16;
        Box::new(PreIndexedIndirectAddressing { address, offset })
    }
}

impl AddressingMode for PreIndexedIndirectAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::PreIndexedIndirect
    }

    fn fetch(&self, mem: &Memory) -> u8 {
        let lsb_location = self.address + (self.offset as u16);
        let lsb = mem.get(lsb_location as usize);
        let msb = mem.get((lsb_location+1) as usize);

        let address = ((msb as u16) << 8) + (lsb as u16);
        mem.get(address as usize)
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        let lsb_location = self.address + (self.offset as u16);
        let lsb = mem.get(lsb_location as usize);
        let msb = mem.get((lsb_location+1) as usize);

        let address = ((msb as u16) << 8) + (lsb as u16);
        mem.set(address as usize, v);
    }
    
    fn debug(&self) -> String {
        format!("Pre-index Indirect adressing at: 0x{:x}+0x{:x}",
                self.address,
                self.offset)
    }
}

// Indirect indexed (aka post-indexed)... wtf.
// E.g. LDA ($44), Y
// --------------------------------------------
pub struct PostIndexedIndirectAddressing {
    address: u16, // address is u16 but is always 0x00XX
    offset: u8,
}

impl PostIndexedIndirectAddressing {
    pub fn new(address_byte: u8, offset: u8) -> Box<PostIndexedIndirectAddressing> {
        let address = address_byte as u16;
        Box::new(PostIndexedIndirectAddressing { address, offset })
    }
}

impl AddressingMode for PostIndexedIndirectAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::PostIndexedIndirect
    }

    fn fetch(&self, mem: &Memory) -> u8 {
        let lsb = mem.get(self.address as usize);
        let msb = mem.get((self.address+1) as usize);

        let address = ((msb as u16) << 8) + (lsb as u16);
        mem.get((address+ (self.offset as u16)) as usize)
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        let lsb = mem.get(self.address as usize);
        let msb = mem.get((self.address+1) as usize);
        let address = ((msb as u16) << 8) + (lsb as u16);

        mem.set((address+(self.offset as u16)) as usize, v);
    }

    fn debug(&self) -> String {
        format!("Post-index Indirect adressing at: 0x{:x}+0x{:x}",
                self.address,
                self.offset)
    }
}

// Accumulator. Return the accumulator directly.
// ---------------------------------------------
pub struct AccumulatorAddressing {
    A: u8,
}

impl AccumulatorAddressing {
    pub fn new(nes: &Nes) -> Box<AccumulatorAddressing> {
        Box::new(AccumulatorAddressing { A: nes.A() })
    }
}

impl AddressingMode for AccumulatorAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Accumulator
    }

    fn fetch(&self, _mem: &Memory) -> u8 {
        self.A
    }

    fn set(&self, _mem: &mut Memory, _v: u8) {
        // exceptional case. A is set directly
        // in cpu.rs
    }

    fn debug(&self) -> String {
        format!("Accumulator adressing : 0x{:x}",
                self.A)
    }
}
// ------------------------------------------------------------------------
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

    #[test]
    fn test_indexed_zero_page_no_wrapping() {
        let mut memory = Memory::new(vec![1, 2 ,3 ,4 ,5]);
        memory.mem[0x02] = 3;
        let addressing = IndexedZeroPageAddressing::new(0x01, 0x01);
        assert_eq!(3, addressing.fetch(&memory));
    }

    #[test]
    fn test_indexed_zero_page_with_wrapping() {
        let mut memory = Memory::new(vec![1, 2 ,3 ,4 ,5]);
        memory.mem[0x02] = 3;
        let addressing = IndexedZeroPageAddressing::new(0xFF, 0x03);
        assert_eq!(3, addressing.fetch(&memory));
    }
    
    #[test]
    fn test_absolute() {
        let mut memory = Memory::new(vec![1, 2 ,3 ,4 ,5]);
        memory.mem[0x21F5] = 3;
        let addressing = AbsoluteAddressing::new(0xF5, 0x21);
        assert_eq!(3, addressing.fetch(&memory));
    }

    #[test]
    fn test_indexed_absolute() {
        let mut memory = Memory::new(vec![1, 2 ,3 ,4 ,5]);
        memory.mem[0x21F5] = 3;
        let addressing = IndexedAbsoluteAddressing::new(0xF2, 0x21, 0x03);
        assert_eq!(3, addressing.fetch(&memory));
    }

    #[test]
    fn test_indirect() {
        let mut memory = Memory::new(vec![1, 2 ,3 ,4 ,5]);
        memory.mem[0x21F5] = 3;
        memory.mem[0x1213] = 0xF5;
        memory.mem[0x1214] = 0x21;
        let addressing = IndirectAddressing::new(0x13, 0x12);
        assert_eq!(3, addressing.fetch(&memory));
    }

    #[test]
    fn test_pre_indexed_indirect() {
        let mut memory = Memory::new(vec![1, 2 ,3 ,4 ,5]);
        memory.mem[0x21F5] = 3;
        memory.mem[0x0013] = 0xF5;
        memory.mem[0x0014] = 0x21;
        let addressing = PreIndexedIndirectAddressing::new(0x11, 0x02);
        assert_eq!(3, addressing.fetch(&memory));
    }
    

    #[test]
    fn test_post_indexed_indirect() {
        let mut memory = Memory::new(vec![1, 2 ,3 ,4 ,5]);
        memory.mem[0x21F5] = 3;
        memory.mem[0x0013] = 0xF3;
        memory.mem[0x0014] = 0x21;
        let addressing = PostIndexedIndirectAddressing::new(0x13, 0x02);
        assert_eq!(3, addressing.fetch(&memory));
    }

}
    
