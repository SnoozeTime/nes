use super::memory::{AddressingMode, RelativeAddressing, create_addressing, AddressingModeType};
use super::cpu::Nes;
use std::fmt;

macro_rules! instructions {

    ($( $name:ident => {$($code:expr => ($other:expr, $cost:expr)),+} ),+) => {

        #[allow(non_snake_case)] 
        pub enum Instruction {
            $($name(u16, Box<dyn AddressingMode>, u8)),+
            ,
            UNKNOWN(u16, u8)
        }

        impl fmt::Debug for Instruction {
           fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

                match self {
                $(
                    Instruction::$name(line, method, cost) => write!(f, "0x{:x}\t{}\t{:?}\tcycles: {}", line, stringify!($name), *method, cost)
                ),+
                ,
                Instruction::UNKNOWN(line, opcode) => write!(f, "0x{:x}\tUnknown opcode: 0x{:x}", line, opcode),
                }
            }
        }

        impl Instruction {
            pub fn decode(nes: &mut Nes) -> Instruction {
                let line = nes.PC();
                let opcode = nes.advance();
                match opcode {
                $(
                    $(
                        $code => Instruction::$name(line,
                                                    create_addressing($other, nes), 
                                                    $cost)
                    ),+
                ),+
                ,
                _ => Instruction::UNKNOWN(line, opcode)
                }
            }

            pub fn get_cycles(&self) -> u8 {
            match &self {
                 $(
                    Instruction::$name(line, method, cost) => method.extra_cycles() + cost
                ),+
                ,
                Instruction::UNKNOWN(..) => 0
            }
            }
        }
    };
}

instructions!{
    // -----------------------------------------------
    // Arithmetic operations perform addition and subtractions
    // on the contents of the accumulator.
    // ---------------------------------------------------
    // ADC add with carry 
    // This instruction adds the contents of a memory location to A together
    // with the carry bit.
    ADC => {
        0x69 => (AddressingModeType::Immediate, 2),
        0x65 => (AddressingModeType::ZeroPage, 3),
        0x75 => (AddressingModeType::ZeroPageX, 4),
        0x6D => (AddressingModeType::Absolute, 4),
        0x7D => (AddressingModeType::AbsoluteX, 4),
        0x79 => (AddressingModeType::AbsoluteY, 4),
        0x61 => (AddressingModeType::PreIndexedIndirect, 6),
        0x71 => (AddressingModeType::PostIndexedIndirect, 5)
    },

    // SBC - Subtract iwth carry.
    // A,Z,C,N = A-M-(1-C)
    SBC => {
        0xE9 => (AddressingModeType::Immediate, 2),
        0xE5 => (AddressingModeType::ZeroPage, 3),
        0xF5 => (AddressingModeType::ZeroPageX, 4),
        0xED => (AddressingModeType::Absolute, 4),
        0xFD => (AddressingModeType::AbsoluteX, 4),
        0xF9 => (AddressingModeType::AbsoluteY, 4),
        0xE1 => (AddressingModeType::PreIndexedIndirect, 6),
        0xF1 => (AddressingModeType::PostIndexedIndirect, 5)
    },

    // CMP - Compare
    // Z,C,N = A - M
    // Carry flag set if A >= M
    // Zero Flag if A = M
    // N
    CMP => {
        0xC9 => (AddressingModeType::Immediate, 2),
        0xC5 => (AddressingModeType::ZeroPage, 3),
        0xD5 => (AddressingModeType::ZeroPageX, 4),
        0xCD => (AddressingModeType::Absolute, 4),
        0xDD => (AddressingModeType::AbsoluteX, 4),
        0xD9 => (AddressingModeType::AbsoluteY, 4),
        0xC1 => (AddressingModeType::PreIndexedIndirect, 6),
        0xD1 => (AddressingModeType::PostIndexedIndirect, 5)
    },

    // CPX - Compare X register
    // Z, C, N = X - M 
    // Same as CMP
    CPX => {
        0xE0 => (AddressingModeType::Immediate, 2),
        0xE4 => (AddressingModeType::ZeroPage, 3),
        0xEC => (AddressingModeType::Absolute, 4)
    },

    // CPY - Compare Y Register
    CPY => {
        0xC0 => (AddressingModeType::Immediate, 2),
        0xC4 => (AddressingModeType::ZeroPage, 3),
        0xCC => (AddressingModeType::Absolute, 4)
    },


    // ------------------------------------------------------
    // Shifts
    // ------------------------------------------------------
    
    // ASL - Arithmetic Shift Left
    // A, Z, C, N = M*2 or M,Z,C,N = M*2 - Same effect as multiplication by 2.
    // Bit 0 is set to 0 and bit 7 is placed in carry. Ignoring 2's complement.
    ASL => {
        0x0A => (AddressingModeType::Accumulator, 2),
        0x06 => (AddressingModeType::ZeroPage, 5),
        0x16 => (AddressingModeType::ZeroPageX, 6),
        0x0E => (AddressingModeType::Absolute, 6),
        0x1E => (AddressingModeType::AbsoluteX, 7)
    },

    // LSR - Logical shift Left
    // A,C,Z,N = A / 2 or M,C,Z,N = M/2
    // Each of the bits in A or M is shift one place to the right. The bit that
    // was in bit 0 is shifted into the carry. bit 7 is 0.
    LSR => {
        0x4A => (AddressingModeType::Accumulator, 2),
        0x46 => (AddressingModeType::ZeroPage, 5),
        0x56 => (AddressingModeType::ZeroPageX, 6),
        0x4E => (AddressingModeType::Absolute, 6),
        0x5E => (AddressingModeType::AbsoluteX, 7)
    },

    // ROL - Rotate Left
    // Shit to the left. Bit 7 is put in carry and old carry is put at bit 0. 
    ROL => {
        0x2A => (AddressingModeType::Accumulator, 2),
        0x26 => (AddressingModeType::ZeroPage, 5),
        0x36 => (AddressingModeType::ZeroPageX, 6),
        0x2E => (AddressingModeType::Absolute, 6),
        0x3E => (AddressingModeType::AbsoluteX, 7)
    },
    
    // ROR - Rotate Right
    // Opposite of ROR
    ROR => {
        0x6A => (AddressingModeType::Accumulator, 2),
        0x66 => (AddressingModeType::ZeroPage, 5),
        0x76 => (AddressingModeType::ZeroPageX, 6),
        0x6E => (AddressingModeType::Absolute, 6),
        0x6E => (AddressingModeType::AbsoluteX, 7)
    },
 
    // BCC - Branch if Carry Clear
    // If the carry flag is clear then add the relative displacement to the program
    // counter to cause a branch to a new location.
    // Relative     $90 2   2(+1 if branch succeeds)
    // This is only Relative Addressing here :)
    BCC => {
        0x90 => (AddressingModeType::Relative, 2)
    },

    // BCS - Branch is carry Set. Opposite of BCC
    // Relative     $B0 2   2(+1 if branch taken)
    BCS => {
        0xB0 => (AddressingModeType::Relative, 2)
    },

   
    // BEQ - Branch if Equal
    // Take the branch if zero flag is set.
    // Relative     $F0 2   2
    BEQ => {
        0xF0 => (AddressingModeType::Relative, 2)
    },

    //---------------------------------------------------
    // Logical. Perform logical operations on the contents
    // of the accumulator and another value in memory
    // --------------------------------------------------
    // AND - A logical AND is performed, bit by bit, on the accumulator content
    // using the contents of a byte in memory.
    AND => {
        0x29 => (AddressingModeType::Immediate, 2),
        0x25 => (AddressingModeType::ZeroPage, 3),
        0x35 => (AddressingModeType::ZeroPageX, 4),
        0x2D => (AddressingModeType::Absolute, 4),
        0x3D => (AddressingModeType::AbsoluteX, 4),
        0x39 => (AddressingModeType::AbsoluteY, 4),
        0x21 => (AddressingModeType::PreIndexedIndirect, 6),
        0x31 => (AddressingModeType::PostIndexedIndirect, 5)
    },

    // EOR - Exclusive OR
    // A,Z,M = A^M
    // An exclusive OR
    EOR => {
        0x49 => (AddressingModeType::Immediate, 2),
        0x45 => (AddressingModeType::ZeroPage, 3),
        0x55 => (AddressingModeType::ZeroPageX, 4),
        0x4D => (AddressingModeType::Absolute, 4),
        0x5D => (AddressingModeType::AbsoluteX, 4),
        0x59 => (AddressingModeType::AbsoluteY, 4),
        0x41 => (AddressingModeType::PreIndexedIndirect, 6),
        0x51 => (AddressingModeType::PostIndexedIndirect, 5)
    },

    // ORA - Logical inclusive OR
    // A,Z,N = A|M
    ORA => {
        0x09 => (AddressingModeType::Immediate, 2),
        0x05 => (AddressingModeType::ZeroPage, 3),
        0x15 => (AddressingModeType::ZeroPageX, 4),
        0x0D => (AddressingModeType::Absolute, 4),
        0x1D => (AddressingModeType::AbsoluteX, 4),
        0x19 => (AddressingModeType::AbsoluteY, 4),
        0x01 => (AddressingModeType::PreIndexedIndirect, 6),
        0x11 => (AddressingModeType::PostIndexedIndirect, 5)
    },

    // BIT - Bit Test
    // The instruction is used to test if one or more bits are set in a target
    // memory location. The mask pattern in A is ANDed with the value in memory to
    // set or clear the zero flag. The result if not kept.
    // Bits 7 and 6 of the value from memory are copied into the N and V flags
    BIT => {
        0x24 => (AddressingModeType::ZeroPage, 3),
        0x2C => (AddressingModeType::Absolute, 4)
    },

    // Increment and decrements
    // Increment and decrement a memory locatin or one of the X or Y registers by one
    // INC - INCrement a memory location
    // M,Z,N = M + 1
    INC => {
        0xE6 => (AddressingModeType::ZeroPage, 5), 
        0xF6 => (AddressingModeType::ZeroPageX, 6), 
        0xEE => (AddressingModeType::Absolute, 6), 
        0xFE => (AddressingModeType::AbsoluteX, 7) 
    },

    // INX - INCrement X
    INX => {
        0xE8 => (AddressingModeType::Implied, 2)
    },

    // INY - INCrement Y
    INY => {
        0xC8 => (AddressingModeType::Implied, 2)
    },

    // DEC - DEcrement value in memory
    DEC => {
        0xC6 => (AddressingModeType::ZeroPage, 5), 
        0xD6 => (AddressingModeType::ZeroPageX, 6), 
        0xCE => (AddressingModeType::Absolute, 6), 
        0xDE => (AddressingModeType::AbsoluteX, 7) 
    },

    DEX => {
        0xCA => (AddressingModeType::Implied, 2)
    },

    DEY => {
        0x88 => (AddressingModeType::Implied, 2)
    },

    // BMI - Branch if minus
    // Take the branch if negative flag is set
    // Relative     $30 2 2
    BMI => {
        0x30 => (AddressingModeType::Relative, 2)
    },

    // BNE - Branch not equal
    // Opposite of BEQ
    // Relative     $D0 2   2
    BNE => {
        0xD0 => (AddressingModeType::Relative, 2)
    },

    // BPL - Branch if positive
    // Opposite of BMI
    // Relative $10 2   2
    BPL => {
        0x10 => (AddressingModeType::Relative, 2)
    },

   
    // BVC - Branch if overflow clear
    // Relative $50 2   2
    BVC => {
        0x50 => (AddressingModeType::Relative, 2)
    },

    // BVS - Branch if overflow set
    // Relative $70 2   2
    BVS => {
        0x70 => (AddressingModeType::Relative, 2)
    },

    // CLC - Clear carry flag.
    // C = 0
    // Implied, we don't need addressing modes.
    // Implied  $18 1   2
    CLC => {
        0x18 => (AddressingModeType::Implied, 2)
    },

    // CLD - Clear Decimal Mode
    // D = 0
    // Implied  $D8 1   2
    CLD => {
        0xD8 => (AddressingModeType::Implied, 2)
    },

    // CLI - Clear Interrupt Disable
    // I = 0
    // Implied $58  1   2
    CLI => {
        0x58 => (AddressingModeType::Implied, 2)
    },

    // CLV - Clear overflow Flag
    // V = 0
    // Implied  $B8 1   2
    CLV => {
        0xB8 => (AddressingModeType::Implied, 2)
    },

    // --------------------------------------------
    // Load/store operations. Transfer a single byte
    // between registers and memory. Loading affect flags
    // N and Z
    // --------------------------------------------
    
    // LDA - LoaD A
    // Line in memory, address mode, price
    LDA => {
        0xA9 => (AddressingModeType::Immediate, 2),
        0xA5 => (AddressingModeType::ZeroPage, 3),
        0xB5 => (AddressingModeType::ZeroPageX, 4),
        0xAD => (AddressingModeType::Absolute, 4),
        0xBD => (AddressingModeType::AbsoluteX, 4),
        0xB9 => (AddressingModeType::AbsoluteY, 4),
        0xA1 => (AddressingModeType::PreIndexedIndirect, 6),
        0xB1 => (AddressingModeType::PostIndexedIndirect, 5)
    },

    // LDX - LoaD X
    LDX => {
        0xA2 => (AddressingModeType::Immediate, 2),
        0xA6 => (AddressingModeType::ZeroPage, 3),
        0xB6 => (AddressingModeType::ZeroPageY, 4),
        0xAE => (AddressingModeType::Absolute, 4),
        0xBE => (AddressingModeType::AbsoluteY, 4)
    },
    
    // LDY - Load Y
    LDY => {
        0xA0 => (AddressingModeType::Immediate, 2),
        0xA4 => (AddressingModeType::ZeroPage, 3),
        0xB4 => (AddressingModeType::ZeroPageX, 4),
        0xAC => (AddressingModeType::Absolute, 4),
        0xBC => (AddressingModeType::AbsoluteX, 4)
    },

    // STA Store accumulator
    // Store content of acc in memory.
    STA => {
        0x85 => (AddressingModeType::ZeroPage, 3),
        0x95 => (AddressingModeType::ZeroPageX, 4),
        0x8D => (AddressingModeType::Absolute, 4),
        0x9D => (AddressingModeType::AbsoluteX, 5),
        0x99 => (AddressingModeType::AbsoluteY, 5),
        0x81 => (AddressingModeType::PreIndexedIndirect, 6),
        0x91 => (AddressingModeType::PostIndexedIndirect, 6)
    },

    // STX Store X register
    // M = X
    STX => {
        0x86 => (AddressingModeType::ZeroPage, 3),
        0x96 => (AddressingModeType::ZeroPageY, 4),
        0x8E => (AddressingModeType::Absolute, 4)
    },

    // STY Store Y register
    // M = Y
    STY => {
        0x84 => (AddressingModeType::ZeroPage, 3),
        0x94 => (AddressingModeType::ZeroPageX, 4),
        0x8C => (AddressingModeType::Absolute, 4)
    },


    // -----------------------------------
    // Register transfer. X and Y can be moved
    // to and from A
    // ------------------------------------
    
    // TAX - Transfer A to X
    // Set N and Z
    // X = A
    TAX => {
        0xAA => (AddressingModeType::Implied, 2)
    },


    // TAY - Transfer A to X
    // Y = A
    TAY => {
        0xA8 => (AddressingModeType::Implied, 2)
    },


    // TXA Transfer X to A
    // A = X
    TXA => {
        0x8A => (AddressingModeType::Implied, 2)
    },


    // TYA transfer Y to A
    // A = Y
    TYA => {
        0x98 => (AddressingModeType::Implied, 2)
    },

    // Stack Operations.
    // The 6502 processor supports a 256 bytes stack fixed between $0100 and $01FF.
    // A special 8-bit register, SP, is used to keep track of the next free byte of space.
    // The Stack is descending. When pusing a byte to the stack, SP is decremented.
    
    // TSX Transfer stack pointer to X.
    // X = S
    // Copies the current contents of the stack register and set Z and N
    TSX => {
        0xBA => (AddressingModeType::Implied, 2)
    },

    // TXS - Transfer X to SP
    // SP = X
    // Copied X into the stack register. Flags not affeected
    TXS => {
        0x9A => (AddressingModeType::Implied, 2)
    },

    // PHA - PusH Accumulator
    // Push a copy of the accumulator on to the stack. will decrease SP.
    // Flags not affected
    PHA => {
        0x48 => (AddressingModeType::Implied, 3)
    },

    // PLA - PulL Accumulator
    // Pulls a value from the stack into the accumulator. Z and N flags impacted
    PLA => {
        0x68 => (AddressingModeType::Implied, 4)
    },
    
    // PHP - Push processor status
    PHP => {
        0x08 => (AddressingModeType::Implied, 3)
    },

    // PLP - Pull processor status.
    // Affect all
    PLP => {
        0x28 => (AddressingModeType::Implied, 4)
    }



}
