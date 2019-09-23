use super::memory::Memory;
use std::fmt;
use std::fmt::Debug;
use super::cpu::Cpu;

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
                         nes: &mut Cpu,
                         memory: &mut Memory) -> Box<AddressingMode> {
    use self::AddressingModeType::*;
    match addressing_type {
        Accumulator => AccumulatorAddressing::new(&nes),
        Implied => ImpliedAddressing::new(),
        Immediate => ImmediateAddressing::new(nes.advance(memory)),
        ZeroPage => ZeroPageAddressing::new(nes.advance(memory)),
        ZeroPageX => IndexedZeroPageAddressing::new(nes.advance(memory), nes.get_regx()),
        ZeroPageY => IndexedZeroPageAddressing::new(nes.advance(memory), nes.get_regy()),
        Relative => RelativeAddressing::new(nes.advance(memory)),
        Absolute => {
            let op1 = nes.advance(memory);
            let op2 = nes.advance(memory);
            AbsoluteAddressing::new(op1, op2)
        },
        AbsoluteX => {
            let op1 = nes.advance(memory);
            let op2 = nes.advance(memory);
            IndexedAbsoluteAddressing::new(op1, op2, nes.get_regx())
        },
        AbsoluteY => {
            let op1 = nes.advance(memory);
            let op2 = nes.advance(memory);
            IndexedAbsoluteAddressing::new(op1, op2, nes.get_regy())
        },
        Indirect => {
            let op1 = nes.advance(memory);
            let op2 = nes.advance(memory);
            IndirectAddressing::new(op1, op2)
        },
        PreIndexedIndirect => {
            let op = nes.advance(memory);
            PreIndexedIndirectAddressing::new(op, nes.get_regx())
        },
        PostIndexedIndirect => {
            let op = nes.advance(memory);
            PostIndexedIndirectAddressing::new(op, nes.get_regy())
        },
        _ => panic!("not implemented"),
    }
}

pub trait AddressingMode {
    fn mode_type(&self) -> AddressingModeType;

    // Will get the value from memory.
    fn fetch(&self, mem: &mut Memory) -> u8;
    fn fetch16(&self, _mem: &mut Memory) -> u16 {    
        return 0;
    }

    // will set the value to memory
    fn set(&self, mem: &mut Memory, value: u8);
    fn address(&self, _mem: &mut Memory) -> u16;

    // return extra cycles when crossing a page
    fn extra_cycles(&self) -> u8 { 0 }

    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Data {{ ... }}")
    }
}

impl fmt::Debug for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
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

    fn fetch(&self, _mem: &mut Memory) -> u8 {
        0
    }

    fn set(&self, _mem: &mut Memory, _v: u8) {}

    fn address(&self, _mem: &mut Memory) -> u16 { 0 }

    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}


impl fmt::Debug for ImpliedAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Implied Addressing")
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

    fn fetch(&self, _mem: &mut Memory) -> u8 {
        // memory super useless in that case.
        self.value
    }

    fn set(&self, _mem: &mut Memory, _v: u8) {}

    fn address(&self, _mem: &mut Memory) -> u16 { 0 }
    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }

}

impl fmt::Debug for ImmediateAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Immediate Addressing -> {}", self.value)
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

    fn fetch(&self, _mem: &mut Memory) -> u8 {
        self.offset
    }

    fn address(&self, _mem: &mut Memory) -> u16 { 0 }
    fn set(&self, _mem: &mut Memory, _v: u8) {}    
    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }


}

impl fmt::Debug for RelativeAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Relative adressing: 0x{:x}", self.offset)
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

    fn fetch(&self, mem: &mut Memory) -> u8 {
        mem.get(self.address as usize)
    }

    fn address(&self, _mem: &mut Memory) -> u16 { self.address as u16 }
    fn set(&self, mem: &mut Memory, v: u8) {
        mem.set(self.address as usize, v);
    }    
    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }


}

impl fmt::Debug for ZeroPageAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Zero-page adressing at: 0x{:x}", self.address)
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

    fn fetch(&self, mem: &mut Memory) -> u8 {
        // Address + offset should always be in the zero-page area. So 0x00FF + 0x0001
        // should be 0x0000 and not 0x0100. This is done here by keeping address and offset
        // as u8.
        mem.get(self.address.wrapping_add(self.offset) as usize)
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        mem.set(self.address.wrapping_add(self.offset) as usize, v);
    }    

    fn address(&self, _mem: &mut Memory) -> u16 { self.address.wrapping_add(self.offset) as u16 }

    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }


}

impl fmt::Debug for IndexedZeroPageAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Indexed Zero-page adressing at: 0x{:x} + 0x{:x}",
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

    fn fetch(&self, mem: &mut Memory) -> u8 {
        mem.get(self.address as usize)
    }

    fn fetch16(&self, _mem: &mut Memory) -> u16 {
        self.address
    }

    fn address(&self, _mem: &mut Memory) -> u16 { self.address }
    fn set(&self, mem: &mut Memory, v: u8) {
        mem.set(self.address as usize, v);
    }    
    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }


}

impl fmt::Debug for AbsoluteAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Absolute adressing at: 0x{:x}", self.address)
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

    fn fetch(&self, mem: &mut Memory) -> u8 {
        let target = self.address.wrapping_add(self.offset as u16);
        mem.get(target as usize)
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        let target = self.address.wrapping_add(self.offset as u16);
        mem.set(target as usize, v)
    }    
    
    fn address(&self, _mem: &mut Memory) -> u16 { self.address.wrapping_add(self.offset as u16) }

    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }

    fn extra_cycles(&self) -> u8 {
        let (_, overflow) = ((self.address & 0xFF) as u8).overflowing_add(self.offset);
        if overflow {
            return 1;
        } else {
            return 0;
        }
    }


}

impl fmt::Debug for IndexedAbsoluteAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Indexed Absolute adressing at: 0x{:x}+0x{:x}",
               self.address,
               self.offset)
    }
}

// Indirect addressing - meh
// Indirect  addressing  takes  two  operands,  forming  a  16-bit  address,  which  identifies  the least significant byte of another address which is where the data can be found. For example if the operands are bb and cc, and ccbb contains xx and ccbb + 1 contains yy, then the real target address is yyxx. 
// NB: Only JMP is using this addressing. It has a bug (yeaaa) so if self.lsb_location
// ends with 0xFF, +1 will not correctly cross the page.
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

    fn fetch(&self, _mem: &mut Memory) -> u8 {
        0    
    }

    fn fetch16(&self, mem: &mut Memory) -> u16 {
        let lsb = mem.get(self.lsb_location as usize);
        let mut next_loc = self.lsb_location + 1;
        if (self.lsb_location & 0xFF) as u8 == 0xFF {
            next_loc = self.lsb_location & 0xFF00;
        }
        let msb = mem.get(next_loc as usize);
        let address = ((msb as u16) << 8) + (lsb as u16);
        address
    }

    fn address(&self, mem: &mut Memory) -> u16 {
        let lsb = mem.get(self.lsb_location as usize);
        let msb = mem.get((self.lsb_location+1) as usize);
        ((msb as u16) << 8) + (lsb as u16)
    }
    
    
    fn set(&self, mem: &mut Memory, v: u8) {
        let address = self.address(mem);
        mem.set(address as usize, v);
    }    
    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }


}

impl fmt::Debug for IndirectAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Indirect adressing at: 0x{:x}",
               self.lsb_location)
    }
}

// Indexed indirect (aka pre-indexed)... wtf.
// Initial address in zero-page
// E.g. LDA ($44, X)
// --------------------------------------------
pub struct PreIndexedIndirectAddressing {
    address: u8, // address is u16 but is always 0x00XX
    offset: u8,
}

impl PreIndexedIndirectAddressing {
    pub fn new(address_byte: u8, offset: u8) -> Box<PreIndexedIndirectAddressing> {
        let address = address_byte;
        Box::new(PreIndexedIndirectAddressing { address, offset })
    }
}

impl AddressingMode for PreIndexedIndirectAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::PreIndexedIndirect
    }

    fn fetch(&self, mem: &mut Memory) -> u8 {
        let address = self.address(mem);
        mem.get(address as usize)
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        let address = self.address(mem);
        mem.set(address as usize, v);
    }
    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }

    fn address(&self, mem: &mut Memory) -> u16 {
        let lsb_location = self.address.wrapping_add(self.offset);
        let lsb = mem.get(lsb_location as usize);
        let msb = mem.get(lsb_location.wrapping_add(1) as usize);

        ((msb as u16) << 8) + (lsb as u16)
    }
 
    fn extra_cycles(&self) -> u8 {
        let (_, overflow) = ((self.address & 0xFF) as u8).overflowing_add(self.offset);
        if overflow {
            return 1;
        } else {
            return 0;
        }
    }

}

impl fmt::Debug for PreIndexedIndirectAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pre-index Indirect adressing at: 0x{:x}+0x{:x}",
               self.address,
               self.offset)
    }
}

// Indirect indexed (aka post-indexed)... wtf.
// E.g. LDA ($44), Y
// --------------------------------------------
pub struct PostIndexedIndirectAddressing {
    address: u8, // address is u16 but is always 0x00XX
    offset: u8,
}

impl PostIndexedIndirectAddressing {
    pub fn new(address_byte: u8, offset: u8) -> Box<PostIndexedIndirectAddressing> {
        let address = address_byte;
        Box::new(PostIndexedIndirectAddressing { address, offset })
    }
}

impl AddressingMode for PostIndexedIndirectAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::PostIndexedIndirect
    }

    fn fetch(&self, mem: &mut Memory) -> u8 {
        let lsb = mem.get(self.address as usize);
        let msb = mem.get(self.address.wrapping_add(1) as usize);

        let address = ((msb as u16) << 8) + (lsb as u16);
        let fetch_addr: u16 = address.wrapping_add(self.offset as u16);
        mem.get(fetch_addr as usize)
    }

    fn address(&self, mem: &mut Memory) -> u16 {
        let lsb = mem.get(self.address as usize);
        let msb = mem.get(self.address.wrapping_add(1) as usize);
        let address = ((msb as u16) << 8) + (lsb as u16);
        address.wrapping_add(self.offset as u16)
    }

    fn set(&self, mem: &mut Memory, v: u8) {
        let fetch_addr = self.address(mem);
        mem.set(fetch_addr as usize, v);
    }

    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }


}

impl fmt::Debug for PostIndexedIndirectAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Post-index Indirect adressing at: 0x{:x}+0x{:x}",
               self.address,
               self.offset)
    }
}

// Accumulator. Return the accumulator directly.
// ---------------------------------------------
pub struct AccumulatorAddressing {
    accumulator: u8,
}

impl AccumulatorAddressing {
    pub fn new(nes: &Cpu) -> Box<AccumulatorAddressing> {
        Box::new(AccumulatorAddressing { accumulator: nes.get_acc() })
    }
}

impl AddressingMode for AccumulatorAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Accumulator
    }

    fn fetch(&self, _mem: &mut Memory) -> u8 {
        self.accumulator
    }

    fn address(&self, _mem: &mut Memory) -> u16 {
        0
    }


    fn set(&self, _mem: &mut Memory, _v: u8) {
        // exceptional case. A is set directly
        // in cpu.rs
    }    
    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }


}

impl fmt::Debug for AccumulatorAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Accumulator Addressing -> {}", self.accumulator)
    }
}

// ------------------------------------------------------------------------
#[cfg(test)]
mod tests {

    use super::*;
    use std::default::Default;


    #[test]
    fn test_immediate() {
        let mut memory: Memory = Default::default();
        let addressing = ImmediateAddressing::new(8);
        assert_eq!(8, addressing.fetch(&mut memory));
    }

    #[test]
    fn test_zero_page() {
        let mut memory: Memory = Default::default();
        memory.set(0x02, 3);
        let addressing = ZeroPageAddressing::new(0x02);
        assert_eq!(3, addressing.fetch(&mut memory));
    }

    #[test]
    fn test_indexed_zero_page_no_wrapping() {
        let mut memory: Memory = Default::default();
        memory.set(0x02, 3);
        let addressing = IndexedZeroPageAddressing::new(0x01, 0x01);
        assert_eq!(3, addressing.fetch(&mut memory));
    }

    #[test]
    fn test_indexed_zero_page_with_wrapping() {
        let mut memory: Memory = Default::default();
        memory.set(0x02, 3);
        let addressing = IndexedZeroPageAddressing::new(0xFF, 0x03);
        assert_eq!(3, addressing.fetch(&mut memory));
    }
   
}

