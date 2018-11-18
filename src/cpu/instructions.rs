use super::memory::{AddressingMode, RelativeAddressing};

pub enum Instruction {
    
    // ADC
    // Add with Carry
    // Immediate     $69    2    2
    // ZeroPage      $65    2    3
    // ZeroPage,X    $75    2    4
    // This instruction adds the contents of a memory location to A together
    // with the carry bit.
    ADC(u16, Box<AddressingMode>, u8),

    // ADD - A logical AND is performed, bit by bit, on the accumulator content
    // using the contents of a byte in memory.
    // Z and N are impacted.
    // Immediate    $29 2   2
    // Zeropage     $25 2   3
    // ZP,X         $35 2   4
    // Absolute     $2D 3   4
    // Absolute,X   $3D 3   4
    // Absolute,Y   $39 3   4
    // (indirect,X) $21 2   6
    // (Indirect),Y $31 2   5
    AND(u16, Box<AddressingMode>, u8),


    // ASL - Arithmetic Shift Left
    // A, Z, C, N = M*2 or M,Z,C,N = M*2 - Same effect as multiplication by 2.
    // Bit 0 is set to 0 and bit 7 is placed in carry. Ignoring 2's complement.
    // Accumulator  $0A 1   2
    // ZeroPage     $06 2   5
    // ZeroPage,X   $16 2   6
    // Absolute     $0E 3   6
    // Absolute,X   $1E 3   7
    ASL(u16, Box<AddressingMode>, u8),

    // BCC - Branch if Carry Clear
    // If the carry flag is clear then add the relative displacement to the program
    // counter to cause a branch to a new location.
    // Relative     $90 2   2(+1 if branch succeeds)
    // This is only Relative Addressing here :)
    BCC(u16, Box<RelativeAddressing>, u8),

    // BCS - Branch is carry Set. Opposite of BCC
    // Relative     $B0 2   2(+1 if branch taken)
    BCS(u16, Box<RelativeAddressing>, u8),
    
    // BEQ - Branch if Equal
    // Take the branch if zero flag is set.
    // Relative     $F0 2   2
    BEQ(u16, Box<RelativeAddressing>, u8),

    // BIT - Bit Test
    // The instruction is used to test if one or more bits are set in a target
    // memory location. The mask pattern in A is ANDed with the value in memory to
    // set or clear the zero flag. The result if not kept.
    // Bits 7 and 6 of the value from memory are copied into the N and V flags
    //
    // ZeroPage $24 2   3
    // Absolute $2C 3   4
    BIT(u16, Box<AddressingMode>, u8),

    // BMI - Branch if minus
    // Take the branch if negative flag is set
    // Relative     $30 2 2
    BMI(u16, Box<RelativeAddressing>, u8),

    // BNE - Branch not equal
    // Opposite of BEQ
    // Relative     $D0 2   2
    BNE(u16, Box<RelativeAddressing>, u8),

    // BPL - Branch if positive
    // Opposite of BMI
    // Relative $10 2   2
    BPL(u16, Box<RelativeAddressing>, u8),
    
    // BVC - Branch if overflow clear
    // Relative $50 2   2
    BVC(u16, Box<RelativeAddressing>, u8),

    // BVS - Branch if overflow set
    // Relative $70 2   2
    BVS(u16, Box<RelativeAddressing>, u8),
    
    // CLC - Clear carry flag.
    // C = 0
    // Implied, we don't need addressing modes.
    // Implied  $18 1   2
    CLC(u16, u8),

    // CLD - Clear Decimal Mode
    // D = 0
    // Implied  $D8 1   2
    CLD(u16, u8),

    // CLI - Clear Interrupt Disable
    // I = 0
    // Implied $58  1   2
    CLI(u16, u8),

    // CLV - Clear overflow Flag
    // V = 0
    // Implied  $B8 1   2
    CLV(u16, u8),

    // --------------------------------------------
    // Load/store operations. Transfer a single byte
    // between registers and memory. Loading affect flags
    // N and Z
    // --------------------------------------------
    
    // Immediate     LDA #$44      $A9  2   2
    // Zero Page     LDA $44       $A5  2   3
    // Zero Page,X   LDA $44,X     $B5  2   4
    // Absolute      LDA $4400     $AD  3   4
    // Absolute,X    LDA $4400,X   $BD  3   4+
    // Absolute,Y    LDA $4400,Y   $B9  3   4+
    // Indirect,X    LDA ($44,X)   $A1  2   6
    // Indirect,Y    LDA ($44),Y   $B1  2   5+
    // Line in memory, address mode, price
    LDA(u16, Box<AddressingMode>, u8),

    // Immediate,      $A2, 2, 2
    // Zero Page,      $A6, 2, 3
    // Zero Page, Y,   $B6, 2, 4
    // Absolute        $AE, 3, 4
    // Absolute,Y      $BE, 3, 4+
    //
    LDX(u16, Box<AddressingMode>, u8),
    
    // Immediate,      $A0, 2, 2
    // Zero Page,      $A4, 2, 3
    // Zero Page,X,   $B4, 2, 4
    // Absolute        $AC, 3, 4
    // Absolute,X      $BC, 3, 4+
    //
    LDY(u16, Box<AddressingMode>, u8),

    // STA Store accumulator
    // Store content of acc in memory.
    // ZeroPage     $85 2   3
    // ZeroPage,X   $95 2   4
    // Absolute     $8D 3   4
    // Absolute,X   $9D 3   5
    // Absolute,Y   $99 3   5
    // (Indirect,X) $81 2   6
    // (Indirect),Y $91 2   6
    STA(u16, Box<AddressingMode>, u8),

    // STX Store X register
    // M = X
    // ZeroPage     $86 2   3
    // ZeroPage,Y   $96 2   4
    // Absolute     $8E 3   4
    STX(u16, Box<AddressingMode>, u8),
    
    // STY Store Y register
    // M = Y
    // ZeroPage     $84 2   3
    // ZeroPage,X   $94 2   4
    // Absolute     $8C 3   4
    STY(u16, Box<AddressingMode>, u8),
    
    // -----------------------------------
    // Register transfer. X and Y can be moved
    // to and from A
    // ------------------------------------
    
    // TAX - Transfer A to X
    // Set N and Z
    // X = A
    // Implied  $AA 1   2
    TAX(u16, u8),

    // TAY - Transfer A to X
    // Y = A
    // Implied  $A8 1   2
    TAY(u16, u8),

    // TXA Transfer X to A
    // A = X
    // Implied  $8A 1   2
    TXA(u16, u8),

    // TYA transfer Y to A
    // A = Y
    // Implied $98  1   2
    TYA(u16, u8),
    // unknown opcode at line
    UNKNOWN(u16),
}

impl Instruction {

    // return debug string
    pub fn repr(&self) -> String {

        match *self {
            // Arithmetic
            Instruction::ADC(addr, ref mode, price) =>
                format!("0x{:x}\tADC\tAdd with Carry - {} - {}", addr, mode.debug(), price),
            Instruction::AND(addr, ref mode, price) =>
                format!("0x{:x}\tAND\tBitwise AND - {} - {}", addr, mode.debug(), price),
            Instruction::ASL(addr, ref mode, price) =>
                format!("0x{:x}\tASL\tArithmetic shift left - {} - {}", addr, mode.debug(), price),
            // Branch instructions
            Instruction::BCC(addr, ref mode, price) =>
                format!("0x{:x}\tBCC\tBranch if Carry Clear - {} - {}", addr, mode.debug(), price),
            Instruction::BCS(addr, ref mode, price) =>
                format!("0x{:x}\tBCS\tBranch if Carry Set - {} - {}", addr, mode.debug(), price),
            Instruction::BEQ(addr, ref mode, price) =>
                format!("0x{:x}\tBEQ\tBranch if Equal - {} - {}", addr, mode.debug(), price),
            Instruction::BIT(addr, ref mode, price) =>
                format!("0x{:x}\tBIT\tBit test - {} - {}", addr, mode.debug(), price),
            Instruction::BMI(addr, ref mode, price) =>
                format!("0x{:x}\tBMI\tBranch if Minus - {} - {}", addr, mode.debug(), price),
            Instruction::BNE(addr, ref mode, price) =>
                format!("0x{:x}\tBNE\tBranch if not Equal - {} - {}", addr, mode.debug(), price),
            Instruction::BPL(addr, ref mode, price) =>
                format!("0x{:x}\tBPL\tBranch if positive - {} - {}", addr, mode.debug(), price),
            Instruction::BVS(addr, ref mode, price) =>
                format!("0x{:x}\tBVS\tBranch if overflow set - {} - {}", addr, mode.debug(), price),
            Instruction::BVC(addr, ref mode, price) =>
                format!("0x{:x}\tBVC\tBranch if overflow clear - {} - {}", addr, mode.debug(), price),
            Instruction::CLC(addr, price) =>
                format!("0x{:x}\tCLC\tClear Carry- {}", addr, price),
            Instruction::CLD(addr, price) =>
                format!("0x{:x}\tCLD\tClear Decimal- {}", addr, price),
            Instruction::CLI(addr, price) =>
                format!("0x{:x}\tCLI\tClear Interrupt disable- {}", addr, price),
            Instruction::CLV(addr, price) =>
                format!("0x{:x}\tCLV\tClear Overflow- {}", addr, price),
                
            // Load instructions.
            Instruction::LDA(addr, ref mode, price) =>
                format!("0x{:x}\tLDA\tLoad A - {} - {}", addr, mode.debug(), price),
            Instruction::LDX(addr, ref mode, price) =>
                format!("0x{:x}\tLDX\tLoad X - {} - {}", addr, mode.debug(), price),
            Instruction::LDY(addr, ref mode, price) =>
                format!("0x{:x}\tLDY\tLoad Y - {} - {}", addr, mode.debug(), price),
            Instruction::STY(addr, ref mode, price) =>
                format!("0x{:x}\tSTY\tStore Y - {} - {}", addr, mode.debug(), price),
            Instruction::STA(addr, ref mode, price) =>
                format!("0x{:x}\tSTA\tStore A - {} - {}", addr, mode.debug(), price),
            Instruction::STX(addr, ref mode, price) =>
                format!("0x{:x}\tSTX\tStore X - {} - {}", addr, mode.debug(), price),

            // Transfer instructions
            Instruction::TAX(addr, price) =>
                format!("0x{:x}\tTAX\t X = A - {}", addr, price),
            Instruction::TAY(addr, price) =>
                format!("0x{:x}\tTAY\t Y = A - {}", addr, price),
            Instruction::TXA(addr, price) =>
                format!("0x{:x}\tTXA\t A = X- {}", addr, price),
            Instruction::TYA(addr, price) =>
                format!("0x{:x}\tTYA\t A = Y - {}", addr, price),

            Instruction::UNKNOWN(addr) => format!("0x{:x}\tUnknown opcode", addr),
            _ => panic!("Check code, should not happen"),
        }
    }
}
