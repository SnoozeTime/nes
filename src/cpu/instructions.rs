use super::memory::AddressingMode;

pub enum Instruction {
    
    // ADC
    // Add with Carry
    // Immediate     $69    2    2
    // ZeroPage      $65    2    3
    // ZeroPage,X    $75    2    4
    // This instruction adds the contents of a memory location to A together
    // with the carry bit.
    ADC(u16, Box<AddressingMode>, u8),
    
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

    // unknown opcode at line
    UNKNOWN(u16),
}

impl Instruction {

    // return debug string
    pub fn repr(&self) -> String {

        match *self {
            // Arithmetic
            Instruction::ADC(addr, ref mode, price) =>
                format!("0x{:x}]tADC\tAdd with Carry - {} - {}", addr, mode.debug(), price),
            // Load instructions.
            Instruction::LDA(addr, ref mode, price) =>
                format!("0x{:x}\tLDA\tLoad A - {} - {}", addr, mode.debug(), price),
            Instruction::LDX(addr, ref mode, price) =>
                format!("0x{:x}\tLDX\tLoad X - {} - {}", addr, mode.debug(), price),
            Instruction::LDY(addr, ref mode, price) =>
                format!("0x{:x}\tLDY\tLoad Y - {} - {}", addr, mode.debug(), price),

            Instruction::UNKNOWN(addr) => format!("0x{:x}\tUnknown opcode", addr),
            _ => panic!("Check code, should not happen"),
        }
    }
}
