use super::cpu::Cpu;
use super::memory::Memory;
use std::fmt;
use std::fmt::Debug;

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

#[derive(Debug)]
pub enum MySavior {
    Implied(ImpliedAddressing),
    ZeroPage(ZeroPageAddressing),
    Immediate(ImmediateAddressing),
    Relative(RelativeAddressing),
    IndexedZeroPage(IndexedZeroPageAddressing),
    Absolute(AbsoluteAddressing),
    IndexedAbsolute(IndexedAbsoluteAddressing),
    Indirect(IndirectAddressing),
    PreIndexedIndirect(PreIndexedIndirectAddressing),
    PostIndexedIndirect(PostIndexedIndirectAddressing),
    Accumulator(AccumulatorAddressing),
}

impl MySavior {
    pub fn new(addressing_type: AddressingModeType, nes: &mut Cpu, memory: &mut Memory) -> Self {
        match addressing_type {
            AddressingModeType::Accumulator => {
                MySavior::Accumulator(AccumulatorAddressing::new(&nes))
            }
            AddressingModeType::Implied => MySavior::Implied(ImpliedAddressing::new()),
            AddressingModeType::Immediate => {
                MySavior::Immediate(ImmediateAddressing::new(nes.advance(memory)))
            }
            AddressingModeType::ZeroPage => {
                MySavior::ZeroPage(ZeroPageAddressing::new(nes.advance(memory)))
            }
            AddressingModeType::ZeroPageX => MySavior::IndexedZeroPage(
                IndexedZeroPageAddressing::new(nes.advance(memory), nes.get_regx()),
            ),
            AddressingModeType::ZeroPageY => MySavior::IndexedZeroPage(
                IndexedZeroPageAddressing::new(nes.advance(memory), nes.get_regy()),
            ),
            AddressingModeType::Relative => {
                MySavior::Relative(RelativeAddressing::new(nes.advance(memory)))
            }
            AddressingModeType::Absolute => {
                let op1 = nes.advance(memory);
                let op2 = nes.advance(memory);
                MySavior::Absolute(AbsoluteAddressing::new(op1, op2))
            }
            AddressingModeType::AbsoluteX => {
                let op1 = nes.advance(memory);
                let op2 = nes.advance(memory);
                MySavior::IndexedAbsolute(IndexedAbsoluteAddressing::new(op1, op2, nes.get_regx()))
            }
            AddressingModeType::AbsoluteY => {
                let op1 = nes.advance(memory);
                let op2 = nes.advance(memory);
                MySavior::IndexedAbsolute(IndexedAbsoluteAddressing::new(op1, op2, nes.get_regy()))
            }
            AddressingModeType::Indirect => {
                let op1 = nes.advance(memory);
                let op2 = nes.advance(memory);
                MySavior::Indirect(IndirectAddressing::new(op1, op2))
            }
            AddressingModeType::PreIndexedIndirect => {
                let op = nes.advance(memory);
                MySavior::PreIndexedIndirect(PreIndexedIndirectAddressing::new(op, nes.get_regx()))
            }
            AddressingModeType::PostIndexedIndirect => {
                let op = nes.advance(memory);
                MySavior::PostIndexedIndirect(PostIndexedIndirectAddressing::new(
                    op,
                    nes.get_regy(),
                ))
            }
            _ => panic!("not implemented"),
        }
    }

    pub fn mode_type(&self) -> AddressingModeType {
        use MySavior::*;
        match *self {
            Implied(ref x) => x.mode_type(),
            ZeroPage(ref x) => x.mode_type(),
            Immediate(ref x) => x.mode_type(),
            Relative(ref x) => x.mode_type(),
            IndexedZeroPage(ref x) => x.mode_type(),
            Absolute(ref x) => x.mode_type(),
            IndexedAbsolute(ref x) => x.mode_type(),
            Indirect(ref x) => x.mode_type(),
            PreIndexedIndirect(ref x) => x.mode_type(),
            PostIndexedIndirect(ref x) => x.mode_type(),
            Accumulator(ref x) => x.mode_type(),
        }
    }

    // Will get the value from memory.
    pub fn fetch(&self, mem: &mut Memory) -> u8 {
        use MySavior::*;
        match *self {
            Implied(ref x) => x.fetch(mem),
            ZeroPage(ref x) => x.fetch(mem),
            Immediate(ref x) => x.fetch(mem),
            Relative(ref x) => x.fetch(mem),
            IndexedZeroPage(ref x) => x.fetch(mem),
            Absolute(ref x) => x.fetch(mem),
            IndexedAbsolute(ref x) => x.fetch(mem),
            Indirect(ref x) => x.fetch(mem),
            PreIndexedIndirect(ref x) => x.fetch(mem),
            PostIndexedIndirect(ref x) => x.fetch(mem),
            Accumulator(ref x) => x.fetch(mem),
        }
    }

    pub fn fetch16(&self, mem: &mut Memory) -> u16 {
        use MySavior::*;
        match *self {
            Absolute(ref x) => x.fetch16(mem),
            Indirect(ref x) => x.fetch16(mem),
            _ => 0,
        }

        //return 0;
    }

    // will set the value to memory
    pub fn set(&self, mem: &mut Memory, value: u8) {
        use MySavior::*;
        match *self {
            Implied(ref x) => x.set(mem, value),
            ZeroPage(ref x) => x.set(mem, value),
            Immediate(ref x) => x.set(mem, value),
            Relative(ref x) => x.set(mem, value),
            IndexedZeroPage(ref x) => x.set(mem, value),
            Absolute(ref x) => x.set(mem, value),
            IndexedAbsolute(ref x) => x.set(mem, value),
            Indirect(ref x) => x.set(mem, value),
            PreIndexedIndirect(ref x) => x.set(mem, value),
            PostIndexedIndirect(ref x) => x.set(mem, value),
            Accumulator(ref x) => x.set(mem, value),
        }
    }

    pub fn address(&self, mem: &mut Memory) -> u16 {
        use MySavior::*;
        match *self {
            Implied(ref x) => x.address(mem),
            ZeroPage(ref x) => x.address(mem),
            Immediate(ref x) => x.address(mem),
            Relative(ref x) => x.address(mem),
            IndexedZeroPage(ref x) => x.address(mem),
            Absolute(ref x) => x.address(mem),
            IndexedAbsolute(ref x) => x.address(mem),
            Indirect(ref x) => x.address(mem),
            PreIndexedIndirect(ref x) => x.address(mem),
            PostIndexedIndirect(ref x) => x.address(mem),
            Accumulator(ref x) => x.address(mem),
        }
    }

    // return extra cycles when crossing a page
    pub fn extra_cycles(&self) -> u8 {
        use MySavior::*;
        match *self {
            IndexedAbsolute(ref x) => x.extra_cycles(),
            PreIndexedIndirect(ref x) => x.extra_cycles(),
            _ => 0,
        }

        //0
    }

    pub fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MySavior::*;
        match *self {
            Implied(ref x) => x.debug_fmt(f),
            ZeroPage(ref x) => x.debug_fmt(f),
            Immediate(ref x) => x.debug_fmt(f),
            Relative(ref x) => x.debug_fmt(f),
            IndexedZeroPage(ref x) => x.debug_fmt(f),
            Absolute(ref x) => x.debug_fmt(f),
            IndexedAbsolute(ref x) => x.debug_fmt(f),
            Indirect(ref x) => x.debug_fmt(f),
            PreIndexedIndirect(ref x) => x.debug_fmt(f),
            PostIndexedIndirect(ref x) => x.debug_fmt(f),
            Accumulator(ref x) => x.debug_fmt(f),
        }
    }
}

// Implied. Nothinig to fetch. All the instruction is implied by opcode
// --------------------------------------------------------------------
pub struct ImpliedAddressing {}
impl ImpliedAddressing {
    pub fn new() -> ImpliedAddressing {
        ImpliedAddressing {}
    }
}

impl ImpliedAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Implied
    }

    fn fetch(&self, _mem: &mut Memory) -> u8 {
        0
    }

    fn set(&self, _mem: &mut Memory, _v: u8) {}

    fn address(&self, _mem: &mut Memory) -> u16 {
        0
    }

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
    pub fn new(value: u8) -> ImmediateAddressing {
        ImmediateAddressing { value }
    }
}

impl ImmediateAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Immediate
    }

    fn fetch(&self, _mem: &mut Memory) -> u8 {
        // memory super useless in that case.
        self.value
    }

    fn set(&self, _mem: &mut Memory, _v: u8) {}

    fn address(&self, _mem: &mut Memory) -> u16 {
        0
    }
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
    pub fn new(offset: u8) -> RelativeAddressing {
        RelativeAddressing { offset }
    }
}

impl RelativeAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Relative
    }

    fn fetch(&self, _mem: &mut Memory) -> u8 {
        self.offset
    }

    fn address(&self, _mem: &mut Memory) -> u16 {
        0
    }
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
    pub fn new(address: u8) -> ZeroPageAddressing {
        ZeroPageAddressing { address }
    }
}

impl ZeroPageAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::ZeroPage
    }

    fn fetch(&self, mem: &mut Memory) -> u8 {
        mem.get(self.address as usize)
    }

    fn address(&self, _mem: &mut Memory) -> u16 {
        self.address as u16
    }
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
    pub fn new(address: u8, offset: u8) -> IndexedZeroPageAddressing {
        IndexedZeroPageAddressing { address, offset }
    }
}

impl IndexedZeroPageAddressing {
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

    fn address(&self, _mem: &mut Memory) -> u16 {
        self.address.wrapping_add(self.offset) as u16
    }

    fn debug_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}

impl fmt::Debug for IndexedZeroPageAddressing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Indexed Zero-page adressing at: 0x{:x} + 0x{:x}",
            self.address, self.offset
        )
    }
}

// Absolute addressing mode. In  absolute  addressing,  the  address  of  the  data  to  operate on  is  specified  by  the  two
// operands supplied, least significant byte first
// ----------------------------------------------------------------
pub struct AbsoluteAddressing {
    address: u16, // Create in new function
}

impl AbsoluteAddressing {
    pub fn new(lsb: u8, msb: u8) -> AbsoluteAddressing {
        let address = ((msb as u16) << 8) + (lsb as u16);
        AbsoluteAddressing { address }
    }
}

impl AbsoluteAddressing {
    fn mode_type(&self) -> AddressingModeType {
        AddressingModeType::Absolute
    }

    fn fetch(&self, mem: &mut Memory) -> u8 {
        mem.get(self.address as usize)
    }

    fn fetch16(&self, _mem: &mut Memory) -> u16 {
        self.address
    }

    fn address(&self, _mem: &mut Memory) -> u16 {
        self.address
    }
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
    pub fn new(lsb: u8, msb: u8, offset: u8) -> IndexedAbsoluteAddressing {
        let address = ((msb as u16) << 8) + (lsb as u16);
        IndexedAbsoluteAddressing { address, offset }
    }
}

impl IndexedAbsoluteAddressing {
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

    fn address(&self, _mem: &mut Memory) -> u16 {
        self.address.wrapping_add(self.offset as u16)
    }

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
        write!(
            f,
            "Indexed Absolute adressing at: 0x{:x}+0x{:x}",
            self.address, self.offset
        )
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
    pub fn new(lsb: u8, msb: u8) -> IndirectAddressing {
        let lsb_location = ((msb as u16) << 8) + (lsb as u16);
        IndirectAddressing { lsb_location }
    }
}

impl IndirectAddressing {
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
        let msb = mem.get((self.lsb_location + 1) as usize);
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
        write!(f, "Indirect adressing at: 0x{:x}", self.lsb_location)
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
    pub fn new(address_byte: u8, offset: u8) -> PreIndexedIndirectAddressing {
        let address = address_byte;
        PreIndexedIndirectAddressing { address, offset }
    }
}

impl PreIndexedIndirectAddressing {
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
        write!(
            f,
            "Pre-index Indirect adressing at: 0x{:x}+0x{:x}",
            self.address, self.offset
        )
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
    pub fn new(address_byte: u8, offset: u8) -> PostIndexedIndirectAddressing {
        let address = address_byte;
        PostIndexedIndirectAddressing { address, offset }
    }
}

impl PostIndexedIndirectAddressing {
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
        write!(
            f,
            "Post-index Indirect adressing at: 0x{:x}+0x{:x}",
            self.address, self.offset
        )
    }
}

// Accumulator. Return the accumulator directly.
// ---------------------------------------------
pub struct AccumulatorAddressing {
    accumulator: u8,
}

impl AccumulatorAddressing {
    pub fn new(nes: &Cpu) -> AccumulatorAddressing {
        AccumulatorAddressing {
            accumulator: nes.get_acc(),
        }
    }
}

impl AccumulatorAddressing {
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
