use super::memory::{AddressingMode, create_addressing};
use super::cpu::Cpu;
use std::fmt;
use cpu::memory::AddressingModeType::*;

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
                    Instruction::$name(line, method, cost) => write!(f, "{:x}\t{}\t{:?}\tcycles: {}", line, stringify!($name), *method, cost)
                ),+
                ,
                Instruction::UNKNOWN(line, opcode) => write!(f, "0x{:x}\tUnknown opcode: 0x{:x}", line, opcode),
                }
            }
        }

        impl Instruction {
            pub fn decode(nes: &mut Cpu) -> Instruction {
                let line = nes.get_pc();
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
                    Instruction::$name(_, method, cost) => method.extra_cycles() + cost
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
        0x69 => (Immediate, 2),
        0x65 => (ZeroPage, 3),
        0x75 => (ZeroPageX, 4),
        0x6D => (Absolute, 4),
        0x7D => (AbsoluteX, 4),
        0x79 => (AbsoluteY, 4),
        0x61 => (PreIndexedIndirect, 6),
        0x71 => (PostIndexedIndirect, 5)
    },

    // SBC - Subtract iwth carry.
    // A,Z,C,N = A-M-(1-C)
    SBC => {
        0xE9 => (Immediate, 2),
        0xE5 => (ZeroPage, 3),
        0xF5 => (ZeroPageX, 4),
        0xED => (Absolute, 4),
        0xFD => (AbsoluteX, 4),
        0xF9 => (AbsoluteY, 4),
        0xE1 => (PreIndexedIndirect, 6),
        0xF1 => (PostIndexedIndirect, 5)
    },

    // CMP - Compare
    // Z,C,N = A - M
    // Carry flag set if A >= M
    // Zero Flag if A = M
    // N
    CMP => {
        0xC9 => (Immediate, 2),
        0xC5 => (ZeroPage, 3),
        0xD5 => (ZeroPageX, 4),
        0xCD => (Absolute, 4),
        0xDD => (AbsoluteX, 4),
        0xD9 => (AbsoluteY, 4),
        0xC1 => (PreIndexedIndirect, 6),
        0xD1 => (PostIndexedIndirect, 5)
    },

    // CPX - Compare X register
    // Z, C, N = X - M 
    // Same as CMP
    CPX => {
        0xE0 => (Immediate, 2),
        0xE4 => (ZeroPage, 3),
        0xEC => (Absolute, 4)
    },

    // CPY - Compare Y Register
    CPY => {
        0xC0 => (Immediate, 2),
        0xC4 => (ZeroPage, 3),
        0xCC => (Absolute, 4)
    },


    // ------------------------------------------------------
    // Shifts
    // ------------------------------------------------------
    
    // ASL - Arithmetic Shift Left
    // A, Z, C, N = M*2 or M,Z,C,N = M*2 - Same effect as multiplication by 2.
    // Bit 0 is set to 0 and bit 7 is placed in carry. Ignoring 2's complement.
    ASL => {
        0x0A => (Accumulator, 2),
        0x06 => (ZeroPage, 5),
        0x16 => (ZeroPageX, 6),
        0x0E => (Absolute, 6),
        0x1E => (AbsoluteX, 7)
    },

    // LSR - Logical shift Left
    // A,C,Z,N = A / 2 or M,C,Z,N = M/2
    // Each of the bits in A or M is shift one place to the right. The bit that
    // was in bit 0 is shifted into the carry. bit 7 is 0.
    LSR => {
        0x4A => (Accumulator, 2),
        0x46 => (ZeroPage, 5),
        0x56 => (ZeroPageX, 6),
        0x4E => (Absolute, 6),
        0x5E => (AbsoluteX, 7)
    },

    // ROL - Rotate Left
    // Shit to the left. Bit 7 is put in carry and old carry is put at bit 0. 
    ROL => {
        0x2A => (Accumulator, 2),
        0x26 => (ZeroPage, 5),
        0x36 => (ZeroPageX, 6),
        0x2E => (Absolute, 6),
        0x3E => (AbsoluteX, 7)
    },
    
    // ROR - Rotate Right
    // Opposite of ROR
    ROR => {
        0x6A => (Accumulator, 2),
        0x66 => (ZeroPage, 5),
        0x76 => (ZeroPageX, 6),
        0x6E => (Absolute, 6),
        0x7E => (AbsoluteX, 7)
    },

    // --------------------------------------------------------------------
    // Jumps and calls
    // The Instructions modify the program counter causing a break to normal
    // sequential execution. the JSR pushes the old PC onto the stack before
    // changing it to the new location. Allowing a subsequent RTS to return
    // to the instruction after the call
    // --------------------------------------------------------------------
    
    // JMP - Sets PC to the address specified by the operand.
    // NB:
    // An original 6502 has does not correctly fetch the target address if the indirect
    // vector falls on a page boundary (e.g. $xxFF where xx is any value from $00 to $FF).
    // In this case fetches the LSB from $xxFF as expected but takes the MSB from $xx00.
    // This is fixed in some later chips like the 65SC02 so for compatibility always
    // ensure the indirect vector is not at the end of the page.
    JMP => {
        0x4C => (Absolute, 3),
        0x6C => (Indirect, 5)
    },

    // The JSR instruction pushes the address (minus one) of the return point on to the stack and
    // then sets the program counter to the target memory address.
    JSR => {
        0x20 => (Absolute, 6)
    },

    // RTS - Return from Subroutine
    RTS => {
        0x60 => (Implied, 6)
    },

    // BCC - Branch if Carry Clear
    // If the carry flag is clear then add the relative displacement to the program
    // counter to cause a branch to a new location.
    // Relative     $90 2   2(+1 if branch succeeds)
    // This is only Relative Addressing here :)
    BCC => {
        0x90 => (Relative, 2)
    },

    // BCS - Branch is carry Set. Opposite of BCC
    // Relative     $B0 2   2(+1 if branch taken)
    BCS => {
        0xB0 => (Relative, 2)
    },

   
    // BEQ - Branch if Equal
    // Take the branch if zero flag is set.
    // Relative     $F0 2   2
    BEQ => {
        0xF0 => (Relative, 2)
    },

    // BMI - Branch if minus
    // Take the branch if negative flag is set
    // Relative     $30 2 2
    BMI => {
        0x30 => (Relative, 2)
    },

    // BNE - Branch not equal
    // Opposite of BEQ
    // Relative     $D0 2   2
    BNE => {
        0xD0 => (Relative, 2)
    },

    // BPL - Branch if positive
    // Opposite of BMI
    // Relative $10 2   2
    BPL => {
        0x10 => (Relative, 2)
    },
   
    // BVC - Branch if overflow clear
    // Relative $50 2   2
    BVC => {
        0x50 => (Relative, 2)
    },

    // BVS - Branch if overflow set
    // Relative $70 2   2
    BVS => {
        0x70 => (Relative, 2)
    },

    //---------------------------------------------------
    // Logical. Perform logical operations on the contents
    // of the accumulator and another value in memory
    // --------------------------------------------------
    // AND - A logical AND is performed, bit by bit, on the accumulator content
    // using the contents of a byte in memory.
    AND => {
        0x29 => (Immediate, 2),
        0x25 => (ZeroPage, 3),
        0x35 => (ZeroPageX, 4),
        0x2D => (Absolute, 4),
        0x3D => (AbsoluteX, 4),
        0x39 => (AbsoluteY, 4),
        0x21 => (PreIndexedIndirect, 6),
        0x31 => (PostIndexedIndirect, 5)
    },

    // EOR - Exclusive OR
    // A,Z,M = A^M
    // An exclusive OR
    EOR => {
        0x49 => (Immediate, 2),
        0x45 => (ZeroPage, 3),
        0x55 => (ZeroPageX, 4),
        0x4D => (Absolute, 4),
        0x5D => (AbsoluteX, 4),
        0x59 => (AbsoluteY, 4),
        0x41 => (PreIndexedIndirect, 6),
        0x51 => (PostIndexedIndirect, 5)
    },

    // ORA - Logical inclusive OR
    // A,Z,N = A|M
    ORA => {
        0x09 => (Immediate, 2),
        0x05 => (ZeroPage, 3),
        0x15 => (ZeroPageX, 4),
        0x0D => (Absolute, 4),
        0x1D => (AbsoluteX, 4),
        0x19 => (AbsoluteY, 4),
        0x01 => (PreIndexedIndirect, 6),
        0x11 => (PostIndexedIndirect, 5)
    },

    // BIT - Bit Test
    // The instruction is used to test if one or more bits are set in a target
    // memory location. The mask pattern in A is ANDed with the value in memory to
    // set or clear the zero flag. The result if not kept.
    // Bits 7 and 6 of the value from memory are copied into the N and V flags
    BIT => {
        0x24 => (ZeroPage, 3),
        0x2C => (Absolute, 4)
    },

    // Increment and decrements
    // Increment and decrement a memory locatin or one of the X or Y registers by one
    // INC - INCrement a memory location
    // M,Z,N = M + 1
    INC => {
        0xE6 => (ZeroPage, 5), 
        0xF6 => (ZeroPageX, 6), 
        0xEE => (Absolute, 6), 
        0xFE => (AbsoluteX, 7) 
    },

    // INX - INCrement X
    INX => {
        0xE8 => (Implied, 2)
    },

    // INY - INCrement Y
    INY => {
        0xC8 => (Implied, 2)
    },

    // DEC - DEcrement value in memory
    DEC => {
        0xC6 => (ZeroPage, 5), 
        0xD6 => (ZeroPageX, 6), 
        0xCE => (Absolute, 6), 
        0xDE => (AbsoluteX, 7) 
    },

    DEX => {
        0xCA => (Implied, 2)
    },

    DEY => {
        0x88 => (Implied, 2)
    },
    // CLC - Clear carry flag.
    // C = 0
    // Implied, we don't need addressing modes.
    // Implied  $18 1   2
    CLC => {
        0x18 => (Implied, 2)
    },

    // CLD - Clear Decimal Mode
    // D = 0
    // Implied  $D8 1   2
    CLD => {
        0xD8 => (Implied, 2)
    },

    // CLI - Clear Interrupt Disable
    // I = 0
    // Implied $58  1   2
    CLI => {
        0x58 => (Implied, 2)
    },

    // CLV - Clear overflow Flag
    // V = 0
    // Implied  $B8 1   2
    CLV => {
        0xB8 => (Implied, 2)
    },

    // SEC - Set Carry Flag
    // C = 1
    SEC => {
        0x38 => (Implied, 2)
    },

    // SED - Set Decimal Flag
    // D = 1
    SED => {
        0xF8 => (Implied, 2)
    },

    // SEI - Set Interrupt Disable
    // I = 1
    SEI => {
        0x78 => (Implied, 2)
    },

    // --------------------------------------------
    // Load/store operations. Transfer a single byte
    // between registers and memory. Loading affect flags
    // N and Z
    // --------------------------------------------
    
    // LDA - LoaD A
    // Line in memory, address mode, price
    LDA => {
        0xA9 => (Immediate, 2),
        0xA5 => (ZeroPage, 3),
        0xB5 => (ZeroPageX, 4),
        0xAD => (Absolute, 4),
        0xBD => (AbsoluteX, 4),
        0xB9 => (AbsoluteY, 4),
        0xA1 => (PreIndexedIndirect, 6),
        0xB1 => (PostIndexedIndirect, 5)
    },

    // LDX - LoaD X
    LDX => {
        0xA2 => (Immediate, 2),
        0xA6 => (ZeroPage, 3),
        0xB6 => (ZeroPageY, 4),
        0xAE => (Absolute, 4),
        0xBE => (AbsoluteY, 4)
    },
    
    // LDY - Load Y
    LDY => {
        0xA0 => (Immediate, 2),
        0xA4 => (ZeroPage, 3),
        0xB4 => (ZeroPageX, 4),
        0xAC => (Absolute, 4),
        0xBC => (AbsoluteX, 4)
    },

    // STA Store accumulator
    // Store content of acc in memory.
    STA => {
        0x85 => (ZeroPage, 3),
        0x95 => (ZeroPageX, 4),
        0x8D => (Absolute, 4),
        0x9D => (AbsoluteX, 5),
        0x99 => (AbsoluteY, 5),
        0x81 => (PreIndexedIndirect, 6),
        0x91 => (PostIndexedIndirect, 6)
    },

    // STX Store X register
    // M = X
    STX => {
        0x86 => (ZeroPage, 3),
        0x96 => (ZeroPageY, 4),
        0x8E => (Absolute, 4)
    },

    // STY Store Y register
    // M = Y
    STY => {
        0x84 => (ZeroPage, 3),
        0x94 => (ZeroPageX, 4),
        0x8C => (Absolute, 4)
    },


    // -----------------------------------
    // Register transfer. X and Y can be moved
    // to and from A
    // ------------------------------------
    
    // TAX - Transfer A to X
    // Set N and Z
    // X = A
    TAX => {
        0xAA => (Implied, 2)
    },


    // TAY - Transfer A to X
    // Y = A
    TAY => {
        0xA8 => (Implied, 2)
    },


    // TXA Transfer X to A
    // A = X
    TXA => {
        0x8A => (Implied, 2)
    },


    // TYA transfer Y to A
    // A = Y
    TYA => {
        0x98 => (Implied, 2)
    },

    // Stack Operations.
    // The 6502 processor supports a 256 bytes stack fixed between $0100 and $01FF.
    // A special 8-bit register, SP, is used to keep track of the next free byte of space.
    // The Stack is descending. When pusing a byte to the stack, SP is decremented.
    
    // TSX Transfer stack pointer to X.
    // X = S
    // Copies the current contents of the stack register and set Z and N
    TSX => {
        0xBA => (Implied, 2)
    },

    // TXS - Transfer X to SP
    // SP = X
    // Copied X into the stack register. Flags not affeected
    TXS => {
        0x9A => (Implied, 2)
    },

    // PHA - PusH Accumulator
    // Push a copy of the accumulator on to the stack. will decrease SP.
    // Flags not affected
    PHA => {
        0x48 => (Implied, 3)
    },

    // PLA - PulL Accumulator
    // Pulls a value from the stack into the accumulator. Z and N flags impacted
    PLA => {
        0x68 => (Implied, 4)
    },
    
    // PHP - Push processor status
    PHP => {
        0x08 => (Implied, 3)
    },

    // PLP - Pull processor status.
    // Affect all
    PLP => {
        0x28 => (Implied, 4)
    },

    // --------------------------------------------
    // System functions.
    // --------------------------------------------
    
    // Force an interruption
    BRK => {
        0x00 => (Implied, 7)
    },

    // No-Operation
    NOP => {
        0xEA => (Implied, 2),

        // Illegals one
        0x1A => (Implied, 2),
        0x3A => (Implied, 2),
        0x5A => (Implied, 2),
        0x7A => (Implied, 2),
        0xDA => (Implied, 2),
        0xFA => (Implied, 2)
    },

    // RTI - Return from Interrupt
    RTI => {
        0x40 => (Implied, 6)
    },

    // -------------------------------------------
    // Below are illegal opcode. They have to be
    // implemented because test rom and some commercial
    // games are using them and they can break the code flow
    // if not found. (e.g. supposed to read 4 bytes but just read 1)
    // ----------------------------------------------------------
    
    // COMBINED INSTRUCTIONS - Combine two instructions in one to
    // go around the compressed size of executables.
    
    // ANC - AND Byte with Accumulator. I
    ANC => {
        0x0B => (Immediate, 2),
        0x2B => (Immediate, 2)
    },

    // AXS
    // AND X register with accumulator and store result in memory.
    AXS => {
        0x87 => (ZeroPage, 3),
        0x97 => (ZeroPageY, 4),
        0x83 => (PreIndexedIndirect, 6),
        0x8F => (Absolute, 4)
    },

    // ARR
    // AND byte with accumulator, then rotate one bit right in A and check bit 5 and
    // 6
    // Similar to AND #i then ROR A, except sets the flags differently. N and Z are normal, but C
    // is bit 6 and V is bit 6 xor bit 5. A fast way to perform signed division by 4 is: CMP #$80;
    // ARR #$FF; ROR. This can be extended to larger powers of two.
    ARR => {
        0x6B => (Immediate, 2)
    },

    // ALR
    // AND byte with accumulator, then shift right one bit in accumulator.
    ALR => {
        0x4B => (Immediate, 2)   
    },

    // LAX 
    // Load accumulator and X register with memory.
    // LDA then TAX
    LAX => {
        0xA7 => (ZeroPage, 3),
        0xB7 => (ZeroPageY, 4),
        0xAF => (Absolute, 4),
        0xBF => (AbsoluteY, 4),
        0xA3 => (PreIndexedIndirect, 6),
        0xB3 => (PostIndexedIndirect, 5)
    },

    // SAX
    //AND X register with accumulator and store result in memory. Status
    // flags: N,Z
    SAX => {
        0x87 => (ZeroPage, 3),
        0x97 => (ZeroPageY, 4),
        0x83 => (PostIndexedIndirect, 6),
        0x8F => (Absolute, 4)
    },

    // RMW (read modify write) instructions
    
    // DCP
    // Equivalent to DEC value then CMP value
    DCP => {
        0xC7 => (ZeroPage, 5),
        0xD7 => (ZeroPageX, 6),
        0xCF => (Absolute, 6),
        0xDF => (AbsoluteX, 7),
        0xDB => (AbsoluteY, 7),
        0xC3 => (PreIndexedIndirect, 8),
        0xD3 => (PostIndexedIndirect, 8)
    },

    // ISC
    // Increase memory then subtract memory from A
    // M = M + 1
    // A = A - M)
    ISC => {
        0xE7 => (ZeroPage, 5),
        0xF7 => (ZeroPageX, 6),
        0xEF => (Absolute, 6),
        0xFF => (AbsoluteX, 7),
        0xFB => (AbsoluteY, 7),
        0xE3 => (PreIndexedIndirect, 8),
        0xF3 => (PostIndexedIndirect, 8)
    },

    // RLA
    // Equivalent to ROL value then AND value, except supporting more addressing modes. LDA #$FF followed by RLA is an efficient way to rotate a variable while also loading it in A.
    RLA => {
        0x27 => (ZeroPage, 5),
        0x37 => (ZeroPageX, 6),
        0x2F => (Absolute, 6),
        0x3F => (AbsoluteX, 7),
        0x3B => (AbsoluteY, 7),
        0x23 => (PreIndexedIndirect, 8),
        0x33 => (PostIndexedIndirect, 8)
    },

    // RRA
    // Equivalent to ROR value then ADC value, except supporting more addressing modes.  Essentially this computes A + value / 2, where value is 9-bit and the division is  rounded up.
    RRA => {
        0x67 => (ZeroPage, 5),
        0x77 => (ZeroPageX, 6),
        0x6F => (Absolute, 6),
        0x7F => (AbsoluteX, 7),
        0x7B => (AbsoluteY, 7),
        0x63 => (PreIndexedIndirect, 8),
        0x73 => (PostIndexedIndirect, 8)
    },

    // SLO
    // Shift left one bit in memory then OR A with mem.
    SLO => {
        0x07 => (ZeroPage, 5),
        0x17 => (ZeroPageX, 6),
        0x0F => (Absolute, 6),
        0x1F => (AbsoluteX, 7),
        0x1B => (AbsoluteY, 7),
        0x03 => (PreIndexedIndirect, 8),
        0x13 => (PostIndexedIndirect, 8)
    },


    // SRE
    // Shift right one bit in memory then EOR with memory
    //     Equivalent to LSR value then EOR value, except supporting more addressing modes. LDA #0 followed by SRE is an efficient way to shift a variable while also loading it in A.
    SRE => {
        0x47 => (ZeroPage, 5),
        0x57 => (ZeroPageX, 6),
        0x4F => (Absolute, 6),
        0x5F => (AbsoluteX, 7),
        0x5B => (AbsoluteY, 7),
        0x43 => (PreIndexedIndirect, 8),
        0x53 => (PostIndexedIndirect, 8)
    },

    // Triple nop. Read a value and od nothing.
    TOP => {
        0x0C => (Absolute, 4),
        0x1C => (AbsoluteX, 4),
        0x3C => (AbsoluteX, 4),
        0x5C => (AbsoluteX, 4),
        0x7C => (AbsoluteX, 4),
        0xDC => (AbsoluteX, 4),
        0xFC => (AbsoluteX, 4)
    },

    // DOUBLE NOP. IM OUTTA HERE
    DOP => {
        0x04 => (ZeroPage, 3),
        0x14 => (ZeroPageX, 4),
        0x34 => (ZeroPageX, 4),
        0x44 => (ZeroPage, 3),
        0x54 => (ZeroPageX, 4),
        0x64 => (ZeroPage, 3),
        0x74 => (ZeroPageX, 4),
        0x80 => (Immediate, 2),
        0x82 => (Immediate, 2),
        0x89 => (Immediate, 2),
        0xC2 => (Immediate, 2),
        0xD4 => (ZeroPageX, 4),
        0xE2 => (Immediate, 2),
        0xF4 => (ZeroPageX, 4)
    }


}
