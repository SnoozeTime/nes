use super::cpu::Nes;

pub enum Instruction {
    
    // Immediate     LDA #$44      $A9  2   2
    // Zero Page     LDA $44       $A5  2   3
    // Zero Page,X   LDA $44,X     $B5  2   4
    // Absolute      LDA $4400     $AD  3   4
    // Absolute,X    LDA $4400,X   $BD  3   4+
    // Absolute,Y    LDA $4400,Y   $B9  3   4+
    // Indirect,X    LDA ($44,X)   $A1  2   6
    // Indirect,Y    LDA ($44),Y   $B1  2   5+
    // 
    // TODO add price.
    LDA(u16, Box<Fn (&mut Nes) -> u8>),

    // unknown opcode at line
    UNKNOWN(u16),
}

impl Instruction {

    // return debug string
    pub fn repr(&self) -> String {

        match *self {
            Instruction::LDA(addr, _) =>
                format!("0x{:x}\tLDA\tLoad A", addr),
            Instruction::UNKNOWN(addr) => format!("0x{:x}\tUnknown opcode", addr),
            _ => String::from("Check code, should not happen"),
        }
    }

    // How long it takes to execute
    fn time(&self) -> u8 {
        match *self {
            Instruction::LDA(..) => 2,
            _ => 0,
        }
    }
}
