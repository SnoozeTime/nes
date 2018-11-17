use super::instructions::Instruction;
use super::memory::*;

pub struct Nes {

    memory: Memory,

    // Program counter. Hold the address of the next instruction to be executed
    PC: u16,

    // stack pointer. from top to bottom.
    SP: u8, 

    // Accumulator. 8 bits register used for arithmetic and logic operation
    A: u8,

    // Index registers
    X: u8,
    Y: u8,

    // Processor status (Flags register)
    P: u8,
}

impl Nes {
   
    // will create a new NES with the given code.
    pub fn new(code: Vec<u8>) -> Nes {
        Nes {
            memory: Memory::new(code),
            PC: 0x8000,
            SP: 0xFF, 
            A: 0,
            X: 0,
            Y: 0,
            P: 0,
        }
    }

    pub fn next(&mut self) -> Result<(), Box<std::error::Error>> {

        let instruction = self.decode();
        println!("{}", instruction.repr());

        match instruction {
            Instruction::LDA(_, addressing, length) =>
                self.A = addressing.fetch(&self.memory),
            _ => {}
        }

        Ok(())
    }
    
    // Decode the next instruction
    fn decode(&mut self) -> Instruction {

     
        let line = self.PC;
        let opcode = self.advance();

        // let instruction_result = match { .... }
        match opcode {

        // -------------------------------------
        // AND operations
        // ------------------------------------
//         0x29 => {
//             let zerop_loc = self.advance();
//         },
//         0x25 => println!("AND operand"),

        // ------------------------------------
        // LoaD Accumulator LDA
        // Affect flags S, Z
        // -----------------------------------
        0xA9 => {
            let operand = self.advance();
            Instruction::LDA(line, ImmediateAddressing::new(operand), 2)
            // self.A = operand;
        },
        // -------------------------------------
        // STX - Store X
        // Affect flags: None
        // ------------------------------------
//        0x86 => {
//            // Zero page
//            let zerop_loc = self.advance();
//            println!("STX Operand - Store X at 0x{:x}", zerop_loc);
//        },
//        0x96 => {
//            // Indexing zero page.
//            let zerop_loc = self.advance();
//            println!("STX Operand,Y - Store X at 0x{:x}+Y", zerop_loc);
//        },
//        0x8E => {
//            // absolute indexing
//            let lsb = self.advance();
//            let msb = self.advance();
//            let loc = ((msb as u16) << 8) | (lsb as u16);
//            println!("STX Operand - Absolute. Store X at 0x{:x}", loc);
//        },
//
        _ => Instruction::UNKNOWN(line),
        }

    }
    // Get next instruction and increment PC
    fn advance(&mut self) -> u8 {
        let code = self.memory.get(self.PC as usize);
        self.PC += 1;
        code
    }
}

#[cfg(test)]
mod tests {

    // get names from outer scope.
    use super::*;


    #[test]
    fn test_LDA_immediate() {
        // Load accumulator. Immediate addressing
        let code = vec![0xA9, 0x36]; 

        let mut nes = Nes::new(code);
        assert_eq!(0x8000, nes.PC);
        nes.next();

        assert_eq!(0x8002, nes.PC);
        assert_eq!(0x36, nes.A);
    }

}
